//! Target-Profile Matrix (lane SP1).
//!
//! Defines the three release profiles for the bundled DuckDB wasm/library
//! artifacts, their expected artifact names, hard size ceilings, and the
//! feature set each includes. Pure metadata + selection logic: per-profile
//! upstream artifact acquisition does not exist yet (see `TODO(UNSUPPORTED)`
//! note at the bottom of this module).
//!
//! Wiring note: `lib.rs` is owned by lane MAX1 during this wave, so this
//! module is NOT yet declared via `mod profile;` in `lib.rs`. It is compiled
//! and exercised by the integration test `tests/profile_matrix.rs` via
//! `#[path]` include. Whoever owns `lib.rs` next adds the one-line
//! `pub mod profile;` declaration.
//!
//! # SHIM-COURT note
//! The court in `tests/profile_matrix.rs` proves non-vacuity of the typed
//! refusal: an unknown feature combination MUST return
//! `ProfileError::UnknownFeature` — never a silent fallback to `Core`.

use std::fmt;

/// Byte constant: 1 MiB = 1024 * 1024 bytes.
const MIB: u64 = 1024 * 1024;

/// Hard size ceilings per profile (bytes).
///
/// These are admission ceilings, not goals: an artifact above its ceiling is
/// a REFUSED build for that profile.
pub mod ceilings {
    use super::MIB;

    /// Core profile: must stay under 24 MiB.
    pub const CORE_MAX_BYTES: u64 = 24 * MIB;
    /// Full (Threaded) profile: expected 28-32 MiB, ceiling 32 MiB.
    pub const FULL_MAX_BYTES: u64 = 32 * MIB;
    /// Enterprise (AllExtensions) profile: expected 38-42 MiB, ceiling 42 MiB.
    pub const ENTERPRISE_MAX_BYTES: u64 = 42 * MIB;
}

/// The three release profiles of the Target-Profile Matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile {
    /// Minimal build: no threads, no ICU, no parquet.
    Core,
    /// Full build: Core + threads.
    Threaded,
    /// Enterprise build: Core + threads + ICU + parquet (all extensions).
    AllExtensions,
}

/// Typed errors for profile selection. Fail-closed: no silent fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// A feature name outside the known set was supplied.
    UnknownFeature(String),
    /// The feature combination is not a modeled profile (e.g. threads
    /// combined with icu but without parquet is not currently a modeled
    /// matrix cell — refused, never coerced).
    UnmodeledCombination { features: Vec<String> },
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::UnknownFeature(feat) => {
                write!(
                    f,
                    "unknown profile feature: {feat:?} (known: core, threads, icu, parquet)"
                )
            }
            ProfileError::UnmodeledCombination { features } => {
                write!(f, "unmodeled profile combination: {features:?}")
            }
        }
    }
}

impl std::error::Error for ProfileError {}

impl Profile {
    /// Expected artifact file name for this profile.
    pub fn artifact_name(&self) -> &'static str {
        match self {
            Profile::Core => "frozen-duckdb-core.wasm",
            Profile::Threaded => "frozen-duckdb-full.wasm",
            Profile::AllExtensions => "frozen-duckdb-enterprise.wasm",
        }
    }

    /// Hard size ceiling for this profile, in bytes.
    pub fn ceiling_bytes(&self) -> u64 {
        match self {
            Profile::Core => ceilings::CORE_MAX_BYTES,
            Profile::Threaded => ceilings::FULL_MAX_BYTES,
            Profile::AllExtensions => ceilings::ENTERPRISE_MAX_BYTES,
        }
    }

    /// Cargo feature names this profile includes (relative to `core`, which
    /// is always implied and therefore never listed).
    pub fn features(&self) -> &'static [&'static str] {
        match self {
            Profile::Core => &[],
            Profile::Threaded => &["threads"],
            Profile::AllExtensions => &["threads", "icu", "parquet"],
        }
    }

    /// Select a profile from a set of Cargo feature names.
    ///
    /// - `[]` or `["core"]` -> `Core`
    /// - `["threads"]` -> `Threaded`
    /// - `["threads", "icu"]` or superset containing threads+icu+parquet ->
    ///   `AllExtensions`
    ///
    /// Anything else (unknown feature names, or partial combinations that do
    /// not name a matrix cell) is a typed error — never a silent fallback to
    /// `Core`.
    pub fn from_features(features: &[&str]) -> Result<Profile, ProfileError> {
        // Normalize: "core" is implied everywhere; reject anything unknown.
        let mut saw_threads = false;
        let mut saw_icu = false;
        let mut saw_parquet = false;
        for f in features {
            match *f {
                "core" => {}
                "threads" => saw_threads = true,
                "icu" => saw_icu = true,
                "parquet" => saw_parquet = true,
                other => return Err(ProfileError::UnknownFeature(other.to_string())),
            }
        }

        let n = usize::from(saw_threads) + usize::from(saw_icu) + usize::from(saw_parquet);
        match n {
            0 => Ok(Profile::Core),
            1 => {
                if saw_threads {
                    Ok(Profile::Threaded)
                    // icu-only / parquet-only cells are not modeled matrix
                    // cells; they fall through to the typed refusal below.
                } else {
                    Err(ProfileError::UnmodeledCombination {
                        features: to_sorted(features),
                    })
                }
            }
            _ => {
                if saw_threads && saw_icu && saw_parquet {
                    Ok(Profile::AllExtensions)
                } else {
                    Err(ProfileError::UnmodeledCombination {
                        features: to_sorted(features),
                    })
                }
            }
        }
    }
}

fn to_sorted(features: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = features.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

// Follow-up work, typed UNSUPPORTED (operator spec, SP1):
//
// TODO(UNSUPPORTED: profile-artifact-acquisition): `ensure_binary` in
// `src/lib.rs` does not yet select its cache/artifact filename by profile —
// profile-specific upstream assets (e.g. `frozen-duckdb-core.wasm` vs
// `frozen-duckdb-enterprise.wasm` release assets) do not exist upstream.
// Integrating `Profile::artifact_name()` into `ensure_binary`'s cache
// filename selection is the named follow-up; until that lands, this module
// is selection logic only and `ensure_binary` continues its current
// (unprofiled) behavior. It must NOT silently default to the Core artifact.
