# openproteo-io

> Working name. See [RENAME-TBD.md](RENAME-TBD.md) for the final-name
> discussion.

> Umbrella of the OpenProteo stack for proteomics raw-file access.
> Vendor readers
> [OpenTFRaw](https://github.com/Sigilweaver/OpenTFRaw) (Thermo),
> [OpenTimsTDF](https://github.com/Sigilweaver/OpenTDF) (Bruker), and
> [OpenWRaw](https://github.com/Sigilweaver/OpenWRaw) (Waters) sit on
> top of shared core
> [openproteo-core](https://github.com/Sigilweaver/OpenProteoCore).
> Columnar storage and analysis lives in
> [ProLance](https://github.com/Sigilweaver/ProLance), a downstream
> consumer.

`openproteo-io` is the umbrella crate that ties together the open Rust
mass-spec parsers:

| Vendor | Format | Crate |
| --- | --- | --- |
| Thermo Fisher | `.raw` (Finnigan) | [`opentfraw`](https://github.com/Sigilweaver/OpenTFRaw) |
| Bruker | timsTOF `.d/` (TDF) | [`opentimstdf`](https://github.com/Sigilweaver/OpenTDF) |
| Waters | MassLynx `.raw/` bundle | [`openwraw`](https://github.com/Sigilweaver/OpenWRaw) |

It re-exports each vendor parser behind a Cargo feature, adds
auto-detection of vendor format from a path, and provides a single
`vendor2mzml` binary that converts any supported input to PSI-MS
mzML 1.1.0 via the canonical writer in
[`openproteo-core`](https://github.com/Sigilweaver/OpenProteoCore).

## Crates

* `crates/openproteo-io` (lib) - feature-gated vendor re-exports plus
  `detect_format` / `convert_to_mzml` helpers.
* `crates/openproteo-io-cli` (bin `vendor2mzml`) - the one-stop CLI.
* `crates/openproteo-io-py` (PyO3) - Python bindings exposing zero-copy
  NumPy views (planned).

## Quick start

```sh
cargo run -p openproteo-io-cli --release -- \
    /path/to/sample.raw /tmp/sample.mzML --indexed
```

## License

Apache-2.0. The vendor crates each carry their own license headers and
upstream attribution notes; this repo only orchestrates them.
