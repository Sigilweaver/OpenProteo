# Releasing the OpenProteo stack

The OpenProteo stack ships as five independently versioned repositories:

- **OpenProteo** (umbrella): `vendor2mzml` CLI, `openproteo_io` Python
  module, `openproteo` Python metapackage, docs site.
- **OpenProteoCore**: shared `openproteo-core` Rust crate.
- **OpenTFRaw**, **OpenTimsTDF**, **OpenWRaw**: per-vendor readers.

Per-repo releases use SemVer tags (`v1.0.4`, `v0.1.0`, ...) on each
repo, cut by that repo's maintainer.

A coordinated **stack release** records "which combination of vendor
versions did we test and ship together". The umbrella `OpenProteo`
repo also uses SemVer for its own tag (`v0.1.0`, `v0.2.0`, ...). Tag
the umbrella when:

- a vendor crate has shipped a new SemVer tag and we want the stack to
  pick it up, or
- the umbrella's own surface (CLI, metapackage) has changed.

## Bump rule for the umbrella

- **patch** (`v0.1.0` -> `v0.1.1`): vendor patch releases, doc-only
  changes, internal fixes.
- **minor** (`v0.1.0` -> `v0.2.0`): vendor minor releases, new CLI
  flags, new Python helpers.
- **major** (`v0.x` -> `v1.0`): CLI / Python metapackage breaking
  change, or removed vendor feature.

## Procedure

From inside the OpenProteo working tree:

```sh
# 1. Review the aggregated notes and the current pin table.
scripts/release-stack.sh

# 2. Update STACK.md to match the new pins.
scripts/release-stack.sh --write-stack-md --apply

# 3. Bump CHANGELOG.md to the new umbrella version, commit.
$EDITOR CHANGELOG.md
git commit -am "chore(release): cut vX.Y.Z"

# 4. Create the annotated tag locally (review with `git show vX.Y.Z`).
scripts/release-stack.sh --name vX.Y.Z --tag --apply

# 5. Push the tag (this triggers release.yml + wheels.yml).
git push origin main
git push origin vX.Y.Z
```

The script refuses to mutate when any of the five sibling working trees
is dirty.

## Artifacts attached on tag push

When `vX.Y.Z` is pushed to OpenProteo:

- `release.yml` builds `vendor2mzml` for linux x86_64/aarch64, macOS
  x86_64/aarch64, and windows x86_64, and attaches archives to the
  GitHub release.
- `wheels.yml` builds `openproteo_io` wheels (and the `openproteo`
  metapackage sdist) and attaches them as workflow artifacts.

No automated publish to crates.io / PyPI is configured; both are
manual operator actions after artifact review.

## What `release-stack.sh` does

- Reads pinned versions from each repo's `Cargo.toml` (the umbrella
  reads `crates/openproteo-io-cli/Cargo.toml`; vendor repos read the
  workspace root).
- Captures each repo's short HEAD SHA.
- Emits Markdown release notes to stdout: pin table + each repo's
  most-recent `CHANGELOG.md` section.
- With `--write-stack-md --apply`, overwrites `STACK.md` with the
  current pin table.
- With `--tag --apply`, creates an annotated tag on `OpenProteo` whose
  message is the aggregated release notes.
- With `--push`, pushes the new tag to `origin`.
- Without `--apply`, every mutating action is a dry-run.

## Flags

| Flag | Effect |
|------|--------|
| `--name vX.Y.Z` | Override the umbrella tag name (default: read from `crates/openproteo-io-cli/Cargo.toml`). |
| `--tag` | Create an annotated tag on OpenProteo HEAD. Requires `--apply` to actually write. |
| `--push` | Push the tag to `origin`. Implies `--tag`. Requires `--apply`. |
| `--apply` | Required for any mutation. Without it, every step is a dry-run. |
| `--write-stack-md` | Overwrite `STACK.md` with the current pin table. Requires `--apply`. |
| `--gate-prolance` | Run `scripts/truthtest-prolance.sh` before tagging. With `--apply`, aborts tag creation on non-zero exit. Without `--apply`, only reports the gate command. |
| `--gate-with-corpus` | Pass `--with-corpus` to the ProLance gate (requires `ProLance/corpus/`). Implies `--gate-prolance`. |
| `--help` | Print usage. |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Dry-run completed, or mutating run succeeded. |
| 1 | Unexpected error (tag already exists, etc.). |
| 2 | Dirty working tree in one of the five repos while `--apply` was set. |
| 3 | Missing sibling repo on disk. |
| 4 | ProLance truth-test gate failed (under `--gate-prolance --apply`). |

## Stack truth test (ProLance gate)

ProLance routes all vendor I/O through `openproteo-io` and owns the
end-to-end coverage that exercises the stack: vendor ingest, mzML
roundtrip, and Lance store read/write. The truth-test gate runs
ProLance's integration tests against the current sibling checkouts so
the release reflects what actually works together.

The default mode runs:

- `cargo build --workspace`
- `cargo test -p prolance-ms --features vendors`
- `cargo test -p prolance-core`

The fixture-gated tests in `prolance-ms` exercise `vendor::ingest` for
Thermo / Bruker / Waters via `openproteo_io::collect` whenever sample
files are present. The `--with-corpus` mode additionally runs
`cargo test --workspace --all-features`, which exercises the full
mzML -> Lance -> mzML roundtrip in `prolance-cli` (requires a
populated `ProLance/corpus/`).

```sh
# Run the gate standalone.
scripts/truthtest-prolance.sh

# Run with the local ProLance corpus (slower, exercises end-to-end
# mzML -> Lance -> mzML).
scripts/truthtest-prolance.sh --with-corpus

# Gate a real release.
scripts/release-stack.sh --name vX.Y.Z --tag --gate-prolance --apply
```

The gate exits with:

| Code | Meaning |
|------|---------|
| 0 | Green - safe to tag. |
| 2 | `ProLance/` (or `ProLance/corpus/` with `--with-corpus`) missing. |
| 3 | `cargo build --workspace` failed in ProLance. |
| 4 | `cargo test` failed in ProLance. |

The gate is opt-in - `release-stack.sh` runs without it by default.
Run it for any release that touches a vendor crate, `openproteo-core`,
or `openproteo-io`.
