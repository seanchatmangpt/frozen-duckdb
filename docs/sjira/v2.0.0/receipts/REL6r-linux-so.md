# REL6r — linux x86_64 libduckdb.so candidate

Standing: **BLOCKED (partial)** — no linux x86_64 `.so` produced within the 45-min timebox.

## Resume inspection (task premise)

- Searched `target/` and `/tmp` for a REL6 linux build tree: **none exists**.
  Only `.so` artifacts found are wasm (`target/wasm-*`/`target/wasm-build/*`).
  `/tmp` has no duckdb dirs. `mdfind` for `libduckdb.so` confirms wasm-only.
- `target/adv4/native-duckdb` is a clean DuckDB source tree; `git log -1` →
  `d54cbbe3d Bump httpfs (#27476)`, matching `SOURCE_PIN`
  `d54cbbe3df17e8e5ce68312f483b4155836112aa` in
  `crates/frozen-duckdb-builder/src/lib.rs:21`. Used as the build subject.
- Conclusion: fresh build required, not a resume.

## Attempt

- Env: docker (colima, server linux/arm64), `--platform linux/amd64` (qemu
  emulation), image `debian:bookworm`, toolchain apt-installed in-container.
- Command: `cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release -DBUILD_UNITTESTS=OFF
  -DBUILD_SHELL=OFF -DBUILD_BENCHMARKS=OFF` then `ninja -j$(nproc) duckdb`
  (script `/tmp/rel6r-build.sh`, log `target/rel6r-linux-build.log`).
- Exit: container killed at background time limit; ninja reported
  `ninja: build stopped: interrupted by user.` (no code-reason failure;
  no compile error observed).

## Last real log line

```
[17/494] Building CXX object extension/core_functions/scalar/list/CMakeFiles/duckdb_core_functions_list.cpp.o
```

16 translation units emitted; ~0.55 ninja targets/min under qemu — projected
completion ~14 h, far outside the timebox. No `libduckdb.so` was linked, so
ELF-magic / `duckdb_library_version` / docker smoke checks were not run.

## Next (recommendation)

Run the same pinned tree on a native linux x86_64 runner (CI `linux/amd64`
runner, or colima VM x86_64 profile) — qemu emulation cannot meet any
realistic lane timebox.
