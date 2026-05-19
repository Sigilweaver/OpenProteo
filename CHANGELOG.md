# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- ProLance truth-test gate. New `scripts/truthtest-prolance.sh` runs
  `cargo build --workspace` and `cargo test --features vendors` (and
  optionally `--with-corpus` for the full mzML -> Lance roundtrip) in
  the sibling ProLance checkout. `scripts/release-stack.sh` gains
  `--gate-prolance` and `--gate-with-corpus` flags that invoke the
  gate before tagging the umbrella; a non-zero exit aborts tag
  creation. Documented in `docs/RELEASE.md`.
- Typed `openproteo_io::Error` enum with `thiserror`-based variants
  (`UnsupportedFormat`, `FeatureDisabled`, `Io`, `Core`, feature-gated
  `Thermo`/`Bruker`/`Waters`, `Mzml`). Replaces `Box<dyn Error>` in
  `convert_to_mzml`, `convert_to_mzml_writer`, `collect`, and internal
  helpers. `openproteo-io-cli::mzml_reader` and `openproteo-io-py`'s
  internal helpers now use the same `openproteo_io::Result` alias.
- Coordinated stack release scheme. `scripts/release-stack.sh` reads
  pinned versions across the five-repo stack, aggregates per-repo
  `CHANGELOG.md` entries into combined release notes, and can create +
  push an annotated SemVer tag on the umbrella (dry-run by default;
  `--apply` gates all mutations).
- `STACK.md` pin-table snapshot of the current stack versions.
- `docs/RELEASE.md` documenting the umbrella SemVer tag scheme,
  per-repo bump rules, and the release procedure.
- `openproteo-io` 0.1.0 lib crate: vendor-feature gates (`thermo`,
  `bruker`, `waters`, `all`), `detect_format()` runtime probe, and
  `convert_to_mzml()` one-shot conversion that defers to the matching
  vendor crate.
- `openproteo-io-cli` 0.1.0 binary `vendor2mzml`: takes an input vendor
  path and an output mzML path, auto-detects the format, supports
  `--indexed`.
- End-to-end smoke test that converts Thermo / Bruker / Waters fixtures
  to mzML when present, skipping silently otherwise.
- `RENAME-TBD.md` flagging this as the working name pending the final
  decision (candidate: `OpenProteo`).

### Fixed

- Silenced unused-import warning for `openproteo_core::SpectrumSource`
  inside `openproteo_io::collect()` when no vendor features are
  enabled.

### Notes

- STRATEGY P0 #1 (route ProLance through `openproteo-io`) marked DONE.
  Shipped in ProLance `develop` commits `aece8f6` (single vendor
  ingester via `openproteo_io::collect`) and `708dbc3` (mzML writer
  delegates to `openproteo-core`).
- STRATEGY P3 #11 (ProLance integration tests as the stack truth test)
  marked DONE via the new `--gate-prolance` flag.
