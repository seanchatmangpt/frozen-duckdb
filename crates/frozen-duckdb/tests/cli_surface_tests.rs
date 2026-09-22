//! CLI argument-surface tripwire (G3, 2026-09-21).
//!
//! Law this guards: every CLI subcommand must parse its own definition —
//! duplicate short flags, missing defaults, or conflicting arguments make
//! clap's debug asserts panic at *parse time* (exit 101), which is how
//! `convert` (`-i` used by both `input` and `input_format`) and `summarize`
//! (`-m` used by both `max_length` and `model`) shipped broken while every
//! compile gate stayed green. `debug_assert()` catches exactly that class
//! at `cargo test` time.

use clap::CommandFactory;
use frozen_duckdb::cli::Cli;

#[test]
fn cli_argument_surface_is_consistent() {
    // debug_assert() consumes the Command, so collect owned subcommand
    // clones first, assert on the parent, then recurse into each child.
    fn assert_surface_ok(cmd: clap::Command, path: &str) {
        let mut owned = cmd;
        owned.build();
        let subs: Vec<(String, clap::Command)> = owned
            .get_subcommands()
            .cloned()
            .map(|s| {
                let name = s.get_name().to_string();
                (name, s)
            })
            .collect();
        owned.debug_assert();
        for (name, sub) in subs {
            let sub_path = if path.is_empty() {
                name
            } else {
                format!("{} {}", path, name)
            };
            assert_surface_ok(sub, &sub_path);
        }
    }

    assert_surface_ok(Cli::command(), "");
}

#[test]
fn every_subcommand_renders_help() {
    let mut cmd = Cli::command();
    cmd.build();
    let subcommands: Vec<String> = cmd
        .get_subcommands()
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|mut sub| {
            // render_help needs &mut; panics on malformed argument metadata.
            let _ = sub.render_help();
            sub.get_name().to_string()
        })
        .collect();

    // The documented surface: 12 commands (G3 docs/sjira/v26.9.21/G3.md).
    let expected = [
        "benchmark",
        "complete",
        "convert",
        "download",
        "embed",
        "filter",
        "flock-setup",
        "info",
        "search",
        "summarize",
        "test",
        "validate-ffi",
    ];
    for name in expected {
        assert!(
            subcommands.iter().any(|s| s == name),
            "missing subcommand `{}` from CLI surface",
            name
        );
    }
}
