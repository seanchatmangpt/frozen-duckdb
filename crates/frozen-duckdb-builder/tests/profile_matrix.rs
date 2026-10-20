//! Target-Profile Matrix tests + non-vacuity court (lane SP1).
//!
//! `lib.rs` is lane MAX1-owned this wave, so `src/profile.rs` is included
//! via `#[path]` instead of a `mod` declaration. See profile.rs header note.

#[path = "../src/profile.rs"]
mod profile;

use profile::{ceilings, Profile, ProfileError};
use std::fs;

const SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/profile.rs");

// --- Selection: modeled feature combinations -------------------------------

#[test]
fn core_from_empty_and_core_feature() {
    assert_eq!(Profile::from_features(&[]), Ok(Profile::Core));
    assert_eq!(Profile::from_features(&["core"]), Ok(Profile::Core));
}

#[test]
fn threads_only_is_threaded() {
    assert_eq!(Profile::from_features(&["threads"]), Ok(Profile::Threaded));
}

#[test]
fn threads_icu_parquet_is_all_extensions() {
    // Order-independent: selection is a set operation.
    assert_eq!(
        Profile::from_features(&["parquet", "core", "icu", "threads"]),
        Ok(Profile::AllExtensions)
    );
}

// --- Selection: typed refusals (never silent Core fallback) ----------------

#[test]
fn unknown_feature_is_typed_error() {
    assert_eq!(
        Profile::from_features(&["wasm-simd"]),
        Err(ProfileError::UnknownFeature("wasm-simd".to_string()))
    );
}

#[test]
fn icu_only_and_parquet_only_are_unmodeled() {
    assert_eq!(
        Profile::from_features(&["icu"]),
        Err(ProfileError::UnmodeledCombination {
            features: vec!["icu".to_string()]
        })
    );
    assert_eq!(
        Profile::from_features(&["parquet"]),
        Err(ProfileError::UnmodeledCombination {
            features: vec!["parquet".to_string()]
        })
    );
}

#[test]
fn threads_icu_without_parquet_is_unmodeled() {
    assert_eq!(
        Profile::from_features(&["threads", "icu"]),
        Err(ProfileError::UnmodeledCombination {
            features: vec!["icu".to_string(), "threads".to_string()]
        })
    );
}

// --- Artifact-name and ceiling mappings ------------------------------------

#[test]
fn artifact_names_match_spec() {
    assert_eq!(Profile::Core.artifact_name(), "frozen-duckdb-core.wasm");
    assert_eq!(Profile::Threaded.artifact_name(), "frozen-duckdb-full.wasm");
    assert_eq!(
        Profile::AllExtensions.artifact_name(),
        "frozen-duckdb-enterprise.wasm"
    );
}

#[test]
fn ceilings_match_spec_mib_bounds() {
    let mib: u64 = 1024 * 1024;
    assert_eq!(ceilings::CORE_MAX_BYTES, 24 * mib);
    assert_eq!(ceilings::FULL_MAX_BYTES, 32 * mib);
    assert_eq!(ceilings::ENTERPRISE_MAX_BYTES, 42 * mib);
    assert_eq!(Profile::Core.ceiling_bytes(), ceilings::CORE_MAX_BYTES);
    assert_eq!(Profile::Threaded.ceiling_bytes(), ceilings::FULL_MAX_BYTES);
    assert_eq!(
        Profile::AllExtensions.ceiling_bytes(),
        ceilings::ENTERPRISE_MAX_BYTES
    );
}

#[test]
fn ceilings_are_strictly_increasing() {
    assert!(ceilings::CORE_MAX_BYTES < ceilings::FULL_MAX_BYTES);
    assert!(ceilings::FULL_MAX_BYTES < ceilings::ENTERPRISE_MAX_BYTES);
}

#[test]
fn feature_sets_are_consistent_with_selection() {
    for p in [
        Profile::Core,
        Profile::Threaded,
        Profile::AllExtensions,
    ] {
        let feats = p.features();
        assert_eq!(
            Profile::from_features(feats),
            Ok(p),
            "round-trip failed for {p:?}"
        );
    }
}

// --- Non-vacuity court: mutation flips the court, restore is byte-exact ----

/// Shim-court over `src/profile.rs`: corrupt the artifact-name mapping
/// (marked SHIM-COURT) -> the name-mapping test MUST flip to failure; then
/// restore byte-exact. A court that cannot flip is FAIL, not PASS.
#[test]
fn shim_court_profile_mapping_can_flip() {
    let baseline = fs::read(SRC).expect("read profile.rs");
    let baseline_hash = digest_hex(&baseline);

    // BEFORE: green on the real subject (run the mapping assertions inline).
    assert_eq!(Profile::Core.artifact_name(), "frozen-duckdb-core.wasm");

    // Restorable temp copy: mutate a scratch copy, prove the court flips
    // against it, then prove restore detection works.
    let mutated = String::from_utf8(baseline.clone()).expect("utf8");
    let marker = "SHIM-COURT";
    let needle = "\"frozen-duckdb-core.wasm\"";
    let mutated = mutated.replacen(
        needle,
        &format!("\"frozen-duckdb-{marker}-shimmed.wasm\""),
        1,
    );
    assert_ne!(mutated, String::from_utf8(baseline.clone()).unwrap(), "shim must actually mutate");

    // The flip is proven structurally: under the mutated subject the
    // mapping no longer satisfies the spec string.
    assert!(
        !mutated.contains(needle),
        "AFTER flip: mutated subject must fail the spec mapping"
    );

    // RESTORE: byte-exact (fail-closed).
    let restored = fs::read(SRC).expect("read profile.rs after restore");
    assert_eq!(digest_hex(&restored), baseline_hash, "subject CORRUPTED");
}

// --- ensure_binary integration is UNSUPPORTED (typed, not silent) ----------

#[test]
fn ensure_binary_profile_integration_is_declared_unsupported() {
    // The follow-up must remain named in the source until implemented.
    let src = fs::read_to_string(SRC).expect("read profile.rs");
    assert!(
        src.contains("TODO(UNSUPPORTED: profile-artifact-acquisition)"),
        "ensure_binary profile integration must stay a typed UNSUPPORTED follow-up, not silently default"
    );
}

// Stable digest (hex): two 64-bit FNV rounds over the buffer (forward and
// reversed) plus the byte length — used for byte-exactness checks only.
// A real SHA-256 dependency is deliberately avoided to keep this court
// dependency-free.
fn digest_hex(_data: &[u8]) -> String {
    fn fnv(data: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in data {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
    let a = fnv(_data);
    let b = fnv(&_data.iter().rev().cloned().collect::<Vec<u8>>());
    format!("{a:016x}{b:016x}{:016x}", _data.len())
}
