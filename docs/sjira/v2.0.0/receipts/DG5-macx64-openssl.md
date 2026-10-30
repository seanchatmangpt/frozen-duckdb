# DG5: mac x86_64 leg — x86_64 OpenSSL via from-source cross-compile

## Failure evidence (tag run 39360386823)

4/5 legs green: linux arm64, linux x86_64, mac arm64 green. Mac x86_64
"Build mega-library" failed ~15 minutes into `make` with undefined x86_64
symbols from `httpfs_httplib_client.cpp.o`:

```
_BIO_ctrl, _BIO_free, ... (BIO/SSL/X509 symbol family)
```

Root cause: the preflight's Rosetta probe (`arch -x86_64 /usr/local/bin/brew
--prefix openssl@3`) found nothing — `/usr/local/bin/brew` does not exist on
arm64 `macos-latest` runners (no Rosetta Homebrew keg). The preflight then
took its designed "fail loud later" path: it exported the **arm64** brew
openssl@3 prefix as `OPENSSL_ROOT_DIR`. CMake FindOpenSSL honored it, the
httpfs extension linked its x86_64 slice against an arm64 keg, and the link
failed at the symbol level (arch mismatch surfaces as undefined symbols, not
an "wrong arch" diagnostic).

## Fix chosen: option (b) — build openssl x86_64 from source

Options considered:

- (a) Bootstrap Rosetta Homebrew under `/usr/local` — real x86_64 keg, but
  5-10 min of install overhead per run and a second Homebrew root to
  maintain. Rejected: heaviest path for the same result.
- (b) `./Configure darwin64-x86_64 --prefix=... no-shared && make && make
  install_sw` into `${RUNNER_TEMP}/openssl-x86_64` — clang `-arch x86_64`
  cross-compiles the openssl source natively on the arm64 runner, no
  Rosetta brew needed, ~3-5 min, result cached across retries via the
  `[ ! -d lib ]` guard. **Chosen**: complete httpfs-capable artifact, no
  second brew root, cheapest correct.
- (c) Drop httpfs from the mac x86_64 extension list — divergent artifact,
  must be KD-registered. Not needed since (b) works.

Implementation (`.github/workflows/build-binaries.yml`, Preflight step,
`x86_64` branch): the Rosetta-probe block is replaced by:

1. `OSSL_VER` from `brew list --versions openssl@3`, formula-revision
   suffix stripped (`3.5.4_1` → `3.5.4`; upstream release tags never carry
   `_N`).
2. `curl` the matching `openssl-${OSSL_VER}.tar.gz` from the openssl
   GitHub release, unpack into `${RUNNER_TEMP}/openssl-src`.
3. `./Configure darwin64-x86_64 --prefix="${RUNNER_TEMP}/openssl-x86_64"
   no-shared`; `make -j$(sysctl -n hw.ncpu)`; `make install_sw`.
4. `OPENSSL_ROOT_DIR=${RUNNER_TEMP}/openssl-x86_64` exported to `$GITHUB_ENV`.

The comment in the workflow now records the run id and the true mechanism
(arm64 prefix export → wrong-arch link), replacing the previous comment that
assumed a Rosetta brew might exist.

## Verification status

- YAML validated: `python3 -c "import yaml; yaml.safe_load(...)"` → OK.
- The workflow is a tag/dispatch-triggered CI surface; the real gate is the
  next `workflow_dispatch` run of `build-binaries.yml` on a clean mac x86_64
  leg: expected green "Build mega-library" with httpfs linked against
  `${RUNNER_TEMP}/openssl-x86_64` (`libssl.a`/`libcrypto.a`, x86_64 slice —
  checkable via `lipo -info`). Not yet executed from this lane; leg outcome
  is the falsifier.
- No extension-list trimming: builder parity intact, no KD divergence
  registered.
