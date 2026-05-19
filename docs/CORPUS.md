# Stack corpus schema

A small, shared schema for the per-repo corpora that back the
OpenProteo stack's conformance suite and benchmarks. The schema lives
here so the four reader repos (OpenTFRaw, OpenTimsTDF, OpenWRaw, and
the OpenProteoCore conformance harness that drives them) can agree on
a single shape without merging their corpora.

> **Legal isolation.** Corpus files stay in their respective repos
> and out of git. The *schema* and the *fetcher tooling* are shared;
> the data is not. A takedown affecting one vendor's corpus does not
> touch the others.

## `sources.json` (per repo)

Each repo keeps its own `scripts/sources.json` describing which PRIDE
projects (or other public archives) it draws files from. The shape:

```json
[
  {
    "vendor": "thermo",
    "instrument": "Q Exactive",
    "format_version": "raw-v66",
    "acquisition_mode": "DDA",
    "accession": "PXD012345",
    "files": ["sample_001.raw"],
    "count": 3,
    "expected_spectrum_count": null,
    "notes": "optional free-text"
  }
]
```

Required fields: `vendor`, `instrument`, `accession`.
Optional fields:

- `format_version`: vendor-format version tag, free-form (e.g.
  `raw-v66`, `tdf-3.0`, `masslynx-4.2`).
- `acquisition_mode`: `DDA`, `DIA`, `PRM`, `SRM`, `MS1`, ...
- `files`: explicit filenames to always download.
- `count`: total target file count - the fetcher auto-fills beyond
  the explicit list by listing the PRIDE FTP directory.
- `expected_spectrum_count`: optional integer used by conformance
  tests to assert iteration parity.
- `notes`: anything else.

Vendor identifiers are lowercase: `thermo`, `bruker`, `waters`.

## `manifest.json` (per repo, generated)

The fetcher writes `corpus/manifest.json` keyed by
`{accession}/{original_filename}`:

```json
{
  "PXD012345/sample_001.raw": {
    "instrument": "Q Exactive",
    "dest_filename": "PXD012345_Q_Exactive_sample_001.raw",
    "size_bytes": 1234567890
  }
}
```

The key intentionally uses the original PRIDE filename so that
re-labeling an instrument in `sources.json` does not invalidate the
manifest.

## Shared fetcher

`OpenProteo/scripts/fetch_corpus.py` is a vendor-agnostic port of
OpenTFRaw's original fetcher. Per-repo wrappers (e.g.
`OpenTFRaw/scripts/fetch_corpus.py`) pass repo-local paths into it.

Limitations:

- Currently downloads *single files* from PRIDE. Vendor formats that
  ship as a directory bundle (Bruker `.d/`, Waters `.raw/`) need a
  recursive-fetch mode that is not yet implemented; their
  `sources.json` files start as empty stubs and will grow once that
  mode lands.
- PRIDE API + FTP fallback only. No S3 / generic-HTTP sources yet.

## Conformance + benchmark consumers

`openproteo-core`'s conformance harness reads spectra via the
`SpectrumSource` trait from whatever corpus paths the calling test
provides; it does not itself touch `sources.json` or `manifest.json`.
Benchmark suites (planned, STRATEGY P2 #8) will read the manifest to
locate files.
