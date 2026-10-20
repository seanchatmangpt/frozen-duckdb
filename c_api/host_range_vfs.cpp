// host_range_vfs.cpp — KD-06 mechanism slice (lane X4)
// Implementation of HostRangeFileSystem. Contract:
// docs/design/host-range-vfs-contract.md

#include "host_range_vfs.h"

#include "duckdb/common/exception.hpp"
#include "duckdb/common/virtual_file_system.hpp"
#include "duckdb/main/database.hpp"

namespace hostvfs {

// ---------------------------------------------------------------------------
// FileHandle
// ---------------------------------------------------------------------------
void HostRangeFileHandle::Close() {
  // Contract `close`: drops the handle; read-only object, nothing to flush.
  size_known_ = false;
}

// ---------------------------------------------------------------------------
// DispatchRead — the single point that crosses into the host.
// ---------------------------------------------------------------------------
void HostRangeFileSystem::DispatchRead(HostRangeFileHandle &h, void *buffer,
                                       uint64_t offset, uint64_t length) {
  if (!read_fn_) {
    throw ::duckdb::IOException("host-range: no read callback registered");
  }
  if (length == 0) {
    // Contract: Length == 0 is InvalidInput (pre-wire).
    throw ::duckdb::InvalidInputException("host-range: zero-length read");
  }
  if (offset > UINT64_MAX - length) {
    throw ::duckdb::OutOfRangeException("host-range: offset+length overflow");
  }

  Status st = read_fn_(ctx_, h.Path().c_str(), offset, length,
                       static_cast<uint8_t *>(buffer));
  switch (st) {
  case Status::Success:
    return;
  case Status::OutOfRange:
    throw ::duckdb::OutOfRangeException(
        "host-range: read past EOF (offset=%llu length=%llu)",
        (unsigned long long)offset, (unsigned long long)length);
  case Status::NetworkDenied:
    throw ::duckdb::IOException("host-range: network denied for %s [%llu,%llu)",
                                h.Path().c_str(), (unsigned long long)offset,
                                (unsigned long long)length);
  case Status::QuotaExceeded:
    throw ::duckdb::IOException("host-range: quota exceeded");
  case Status::InvalidInput:
  default:
    throw ::duckdb::InvalidInputException(
        "host-range: invalid input (offset=%llu length=%llu)",
        (unsigned long long)offset, (unsigned long long)length);
  }
}

// ---------------------------------------------------------------------------
// ResolveSize — size_fn if present, else 1-byte range probe (lazy, contract:
// size is never required before read; result cached per handle).
// ---------------------------------------------------------------------------
uint64_t HostRangeFileSystem::ResolveSize(HostRangeFileHandle &h) {
  if (h.SizeKnown()) {
    return h.Size();
  }
  if (!size_fn_) {
    throw ::duckdb::IOException(
        "host-range: no size_fn registered for %s — size is required before "
        "the parquet scanner will proceed (mechanism slice: no faked size)",
        h.Path().c_str());
  }
  uint64_t s = size_fn_(ctx_, h.Path().c_str());
  h.SetSize(s);
  return s;
}

// ---------------------------------------------------------------------------
// OpenFile — contract `open`: resolves the scheme; no I/O on the wire.
// ---------------------------------------------------------------------------
::duckdb::unique_ptr<::duckdb::FileHandle> HostRangeFileSystem::OpenFile(
    const ::duckdb::string &path, ::duckdb::FileOpenFlags flags,
    ::duckdb::optional_ptr<::duckdb::FileOpener> opener) {
  (void)opener;
  if (!CanHandleFile(path)) {
    throw ::duckdb::InvalidInputException(
        "host-range: path not under host-range prefix: %s", path.c_str());
  }
  // Read-only object: reject write-mode opens (contract `write` parity).
  if (flags.OpenForWriting()) {
    throw ::duckdb::InvalidInputException(
        "host-range: object is read-only: %s", path.c_str());
  }
  return ::duckdb::unique_ptr<HostRangeFileHandle>(
      new HostRangeFileHandle(*this, path, flags));
}

// ---------------------------------------------------------------------------
// Reads
// ---------------------------------------------------------------------------
void HostRangeFileSystem::Read(::duckdb::FileHandle &handle, void *buffer,
                               int64_t nr_bytes, idx_t location) {
  auto &h = static_cast<HostRangeFileHandle &>(handle);
  if (nr_bytes < 0) {
    throw ::duckdb::InvalidInputException("host-range: negative length");
  }
  DispatchRead(h, buffer, static_cast<uint64_t>(location),
               static_cast<uint64_t>(nr_bytes));
}

int64_t HostRangeFileSystem::Read(::duckdb::FileHandle &handle, void *buffer,
                                  int64_t nr_bytes) {
  auto &h = static_cast<HostRangeFileHandle &>(handle);
  if (nr_bytes < 0) {
    throw ::duckdb::InvalidInputException("host-range: negative length");
  }
  DispatchRead(h, buffer, h.SeekPosition(),
               static_cast<uint64_t>(nr_bytes));
  return nr_bytes;
}

// ---------------------------------------------------------------------------
// Writes — interface parity only; every write is InvalidInput.
// ---------------------------------------------------------------------------
void HostRangeFileSystem::Write(::duckdb::FileHandle &handle, void *buffer,
                                int64_t nr_bytes, idx_t location) {
  (void)handle;
  (void)buffer;
  (void)nr_bytes;
  (void)location;
  throw ::duckdb::InvalidInputException(
      "host-range: write on a read-only object");
}

int64_t HostRangeFileSystem::Write(::duckdb::FileHandle &handle, void *buffer,
                                   int64_t nr_bytes) {
  (void)handle;
  (void)buffer;
  (void)nr_bytes;
  throw ::duckdb::InvalidInputException(
      "host-range: write on a read-only object");
}

// ---------------------------------------------------------------------------
// Size / existence / glob
// ---------------------------------------------------------------------------
int64_t HostRangeFileSystem::GetFileSize(::duckdb::FileHandle &handle) {
  auto &h = static_cast<HostRangeFileHandle &>(handle);
  return static_cast<int64_t>(ResolveSize(h));
}

bool HostRangeFileSystem::FileExists(const ::duckdb::string &filename,
                                     ::duckdb::optional_ptr<::duckdb::FileOpener> opener) {
  (void)opener;
  if (!CanHandleFile(filename)) {
    return false;
  }
  // Existence = a 1-byte range probe at offset 0 succeeds (no unbounded GET).
  if (!read_fn_) {
    return false;
  }
  uint8_t probe = 0;
  return read_fn_(ctx_, filename.c_str(), 0, 1, &probe) == Status::Success;
}

bool HostRangeFileSystem::CanHandleFile(const ::duckdb::string &fpath) {
  // Dispatch on scheme match (contract: the prefix is internal; the shim
  // rewrites the scheme before the scanner sees the path).
  const char *kPrefix = "host-range://";
  const size_t n = std::char_traits<char>::length(kPrefix);
  return fpath.size() >= n && fpath.compare(0, n, kPrefix) == 0;
}

::duckdb::vector<::duckdb::OpenFileInfo>
HostRangeFileSystem::Glob(const ::duckdb::string &path, ::duckdb::FileOpener *opener) {
  (void)opener;
  if (CanHandleFile(path) && FileExists(path)) {
    return ::duckdb::vector<::duckdb::OpenFileInfo>{::duckdb::OpenFileInfo(path)};
  }
  return {};
}

// ---------------------------------------------------------------------------
// Registration factory — wire the subsystem into the database's VFS.
// ---------------------------------------------------------------------------
HostRangeFileSystem *
RegisterHostRangeFileSystem(::duckdb::DuckDB &db,
                            ::duckdb::unique_ptr<HostRangeFileSystem> fs) {
  HostRangeFileSystem *raw = fs.get();
  ::duckdb::FileSystem &vfs = db.GetFileSystem();
  vfs.RegisterSubSystem(std::move(fs));
  return raw;
}

} // namespace hostvfs
