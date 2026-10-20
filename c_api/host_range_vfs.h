// host_range_vfs.h — KD-06 mechanism slice (lane X4)
//
// A DuckDB FileSystem answering the `host-range://` prefix. All reads are
// dispatched to a host-supplied callback:
//
//   host_range_read_fn(void* ctx, const char* path, uint64_t offset,
//                      uint64_t length, uint8_t* out_buf) -> Status
//
// Statuses follow the closed set in
// docs/design/host-range-vfs-contract.md (Status vocabulary).
//
// Standing: PARTIAL_ALIVE (mechanism slice). Compiles against the pinned
// DuckDB 1.5.5 C++ headers at target/wasm-build/duckdb/src/include.
// The full GATE-V-05 remote-parquet court is OPEN — see host_range_vfs_test.md.

#ifndef FROZEN_DUCKDB_HOST_RANGE_VFS_H
#define FROZEN_DUCKDB_HOST_RANGE_VFS_H

#include "duckdb/common/file_system.hpp"

#include <cstdint>

namespace duckdb {
class DuckDB; // main/database.hpp — full type needed only by the .cpp factory
}

namespace hostvfs {

// ---------------------------------------------------------------------------
// Status vocabulary (closed set — contract law; never a bare negative int).
// ---------------------------------------------------------------------------
enum class Status : uint32_t {
  Success = 0,
  InvalidInput = 1,  // engine (pre-wire): length 0, write on read-only object
  OutOfRange = 2,    // offset + length > object size (or server 416)
  NetworkDenied = 3, // transport refused / policy refusal / cancellation
  QuotaExceeded = 4, // host-side byte/count budget exhausted
};

// Host callback, per contract formal signature. On Success the callee has
// written exactly `length` bytes into out_buf; partial fills are escalated
// by the host before it returns (contract: no partial fills at this layer).
using read_fn_t = Status (*)(void *ctx, const char *path, uint64_t offset,
                             uint64_t length, uint8_t *out_buf);

// Optional size callback. Contract: size/open may be issued in either order;
// the VFS must not require size before read. If size_fn is null, size is
// resolved lazily with a 1-byte range probe through read_fn_.
using size_fn_t = uint64_t (*)(void *ctx, const char *path);

class HostRangeFileSystem;

// FileHandle carrying the host-range path and the lazily-cached size.
class HostRangeFileHandle : public ::duckdb::FileHandle {
public:
  HostRangeFileHandle(::duckdb::FileSystem &fs, std::string path,
                      ::duckdb::FileOpenFlags flags)
      : ::duckdb::FileHandle(fs, path, flags), path_(std::move(path)),
        size_known_(false), size_(0) {}

  void Close() override; // contract: read-only object, no-op bookkeeping

  const std::string &Path() const { return path_; }
  bool SizeKnown() const { return size_known_; }
  uint64_t Size() const { return size_; }
  void SetSize(uint64_t s) {
    size_known_ = true;
    size_ = s;
  }

private:
  std::string path_;
  bool size_known_;
  uint64_t size_;
};

// ---------------------------------------------------------------------------
// HostRangeFileSystem — dispatches every read to the host callback.
// ---------------------------------------------------------------------------
class HostRangeFileSystem : public ::duckdb::FileSystem {
public:
  HostRangeFileSystem(read_fn_t read_fn, void *ctx, size_fn_t size_fn = nullptr)
      : read_fn_(read_fn), ctx_(ctx), size_fn_(size_fn) {}

  // --- FileSystem overrides (the read/write/open surface) ---
  ::duckdb::unique_ptr<::duckdb::FileHandle> OpenFile(
      const ::duckdb::string &path, ::duckdb::FileOpenFlags flags,
      ::duckdb::optional_ptr<::duckdb::FileOpener> opener = nullptr) override;

  // Positional read (the one the parquet scanner uses: footer-first, pages).
  void Read(::duckdb::FileHandle &handle, void *buffer, int64_t nr_bytes,
            ::duckdb::idx_t location) override;

  // Sequential read — advances position. Backed by the positional read.
  int64_t Read(::duckdb::FileHandle &handle, void *buffer,
               int64_t nr_bytes) override;

  // Interface parity only: the remote object is read-only. Every write
  // throws InvalidInput (contract: write on a read-only object).
  void Write(::duckdb::FileHandle &handle, void *buffer, int64_t nr_bytes,
             ::duckdb::idx_t location) override;
  int64_t Write(::duckdb::FileHandle &handle, void *buffer,
                int64_t nr_bytes) override;

  // Size via size_fn (cached per handle); loud failure when absent.
  int64_t GetFileSize(::duckdb::FileHandle &handle) override;

  bool FileExists(const ::duckdb::string &filename,
                  ::duckdb::optional_ptr<::duckdb::FileOpener> opener =
                      nullptr) override;

  // The VFS dispatches on scheme match; only host-range:// paths are ours.
  bool CanHandleFile(const ::duckdb::string &fpath) override;

  bool DirectoryExists(const ::duckdb::string &, ::duckdb::optional_ptr<::duckdb::FileOpener> opener = nullptr) override {
    return false;
  }
  bool IsPipe(const ::duckdb::string &, ::duckdb::optional_ptr<::duckdb::FileOpener> opener = nullptr) override {
    return false;
  }

  ::duckdb::vector<::duckdb::OpenFileInfo>
  Glob(const ::duckdb::string &path, ::duckdb::FileOpener *opener = nullptr) override;

  std::string GetName() const override { return "HostRangeFileSystem"; }

  // Seek support: reads are positional through the callback.
  bool CanSeek() override { return true; }
  bool OnDiskFile(::duckdb::FileHandle &handle) override { return false; }
  ::duckdb::timestamp_t
  GetLastModifiedTime(::duckdb::FileHandle &handle) override {
    return ::duckdb::timestamp_t::ninfinity();
  }
  ::duckdb::FileType GetFileType(::duckdb::FileHandle &handle) override {
    return ::duckdb::FileType::FILE_TYPE_REGULAR;
  }

  read_fn_t read_fn() const { return read_fn_; }
  void *ctx() const { return ctx_; }

private:
  // Shared dispatch: validates input, calls read_fn_, maps Status to
  // exceptions per the closed-set law. Never a silent-zero return.
  void DispatchRead(HostRangeFileHandle &h, void *buffer, uint64_t offset,
                    uint64_t length);

  // size_fn if present (cached per handle); else fails loudly — the
  // mechanism slice does not fake a size from a probe.
  uint64_t ResolveSize(HostRangeFileHandle &h);

  read_fn_t read_fn_;
  void *ctx_;
  size_fn_t size_fn_;
};

// ---------------------------------------------------------------------------
// Registration factory (contract: idempotent registration is out of scope for
// the mechanism slice — this wires one instance into the database's virtual
// file system under the caller-chosen prefix dispatch).
//
//   auto fs = make_unique<HostRangeFileSystem>(read_cb, ctx);
//   RegisterHostRangeFileSystem(db, std::move(fs));
//
// Returns the registered raw pointer (ownership stays with the VFS).
// ---------------------------------------------------------------------------
HostRangeFileSystem *
RegisterHostRangeFileSystem(::duckdb::DuckDB &db,
                            ::duckdb::unique_ptr<HostRangeFileSystem> fs);

} // namespace hostvfs

#endif // FROZEN_DUCKDB_HOST_RANGE_VFS_H
