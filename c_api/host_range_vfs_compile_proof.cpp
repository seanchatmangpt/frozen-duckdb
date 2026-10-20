// host_range_vfs_compile_proof.cpp — compile proof for KD-06 (lane X4).
// Includes the vendored pinned headers + the slice headers and instantiates
// the class so the override table is type-checked (not just parsed).
#include "duckdb.hpp"
#include "host_range_vfs.h"

// Static sanity: the callback signature is exactly the contract's.
static hostvfs::Status contract_read_fn(void *, const char *, uint64_t,
                                        uint64_t, uint8_t *) {
  return hostvfs::Status::Success;
}

int touch() {
  hostvfs::read_fn_t rf = &contract_read_fn;
  hostvfs::HostRangeFileSystem fs(rf, nullptr, nullptr);
  return fs.GetName() == "HostRangeFileSystem" ? 0 : 1;
}
