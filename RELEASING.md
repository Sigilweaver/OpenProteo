# Releasing OpenProteo

Standard operating procedure for cutting an OpenProteo release. This repo
ships two PyPI packages and one crate from a single tag:

| Artifact | Kind | Built by | Published to |
| --- | --- | --- | --- |
| `openproteo-io` | Rust crate | `cargo publish` | crates.io |
| `openproteo-io` | maturin wheel + sdist | `crates/openproteo-io-py` | PyPI |
| `openproteo` | setuptools sdist + wheel (metapackage/facade) | `python/` | PyPI |

The facade (`openproteo`) depends on the binding (`openproteo-io`) plus
optional per-vendor readers (`opentfraw`, `opentimstdf`, `openwraw`).

`scripts/release-stack.sh` is the coordinated-release helper: it reads the
pinned stack versions, aggregates CHANGELOG notes, refreshes `STACK.md`,
and (with `--apply`) creates and pushes the umbrella tag. Run it dry-run
first (no `--apply`) to preview.

---

## Why this SOP exists

Every avoidable release problem in this suite has come from one of a small
set of causes. The checklist below is built to catch each one:

- A package shipped **wheels only, no sdist** - conda-forge builds from
  source, so it could not be packaged.
- An sdist shipped **without a LICENSE** - conda-forge requires the license
  file in the source; the recipe cannot be accepted without it.
- A **wheel-matrix leg failed on a transient runner flake** (Windows
  `curl`/HTTP2 while fetching a crate; macOS SDK download) and silently
  **skipped the PyPI publish**, leaving crates.io ahead of PyPI.
- **Version drift** between `versions.toml`, `STACK.md`, PyPI, and
  crates.io.
- A scary-looking `include_bytes!` / build dependency was **assumed** to
  break source builds without being **tested** (it was behind a default-off
  feature and did not).

---

## Pre-flight

1. **Pick the version.** Bump `[workspace.package] version` in `Cargo.toml`
   and the hardcoded `version` in **both** pyprojects
   (`crates/openproteo-io-py/pyproject.toml` and `python/pyproject.toml`).
   Keep all three in lockstep.
   - The version must not already exist on crates.io or PyPI (publishes are
     irreversible; you cannot overwrite or re-upload).
2. **Reconcile versions.** Confirm the target is consistent with the intent
   in `versions.toml` (ops repo). Regenerate `STACK.md` with
   `scripts/release-stack.sh --write-stack-md --apply`.
3. **CHANGELOG.** Add a dated entry under a new version heading.

## Packaging invariants (the conda-forge gate)

For **each** published Python package, before tagging:

4. **sdist exists.** `openproteo-io` has a `build-sdist` job; the facade is
   built with `python -m build --sdist`. Both must produce a `.tar.gz`.
5. **LICENSE is in the sdist.** The repo `LICENSE` lives at the root, but
   each package is built from a subdirectory, so a copy must be co-located:
   - `openproteo-io` (maturin): `crates/openproteo-io-py/LICENSE` plus
     `include = [{ path = "LICENSE", format = "sdist" }]` under
     `[tool.maturin]`. maturin forbids `..` in `include`, hence the copy.
   - `openproteo` (setuptools): `python/LICENSE`. setuptools auto-includes
     `LICEN[CS]E*` next to `pyproject.toml`; verify it lands in the sdist.
6. **Build and install from source in a clean env.** Do not assume - test:

   ```sh
   # binding
   ( cd crates/openproteo-io-py && uvx maturin sdist --out /tmp/op-io )
   tar tzf /tmp/op-io/*.tar.gz | grep -i licen        # LICENSE present?
   uv venv /tmp/v1 && uv pip install --python /tmp/v1/bin/python \
       --no-binary :all: /tmp/op-io/*.tar.gz
   /tmp/v1/bin/python -c "import openproteo_io"

   # facade
   python -m build --sdist --outdir /tmp/op python/
   tar tzf /tmp/op/*.tar.gz | grep -i licen
   ```

   If a package embeds a generated data file via `include_bytes!` or a
   build script, check whether that code path is actually compiled in the
   published artifact (feature gates, `cfg`) before concluding the sdist is
   broken.

## Release

7. **Tag.** `scripts/release-stack.sh --name vX.Y.Z --tag --push --apply`
   (or tag `vX.Y.Z` by hand). `publish.yml` triggers on `v*` and runs:
   crate publish (`continue-on-error`), openproteo-io wheels + sdist +
   publish, and the openproteo metapackage build + publish.
8. **Watch the run.** A transient wheel-leg failure will skip
   `publish-openproteo-io` (it `needs: [build-wheels, build-sdist]`). This
   is almost always a runner network flake, not a real error. Fix:

   ```sh
   gh run rerun <run-id> --failed
   ```

   Re-running the failed leg lets the skipped publish proceed. The crate
   publish is `continue-on-error`, so a re-tag will not fail on an
   already-published crate. (The facade publish only `needs:
   build-metapackage`, so a wheel flake does not block it.)

## Post-release

9. **Verify on PyPI.** For each package, confirm the new version has an
   sdist and that the sdist contains `LICENSE`:

   ```sh
   python - <<'PY'
   import json, urllib.request, io, tarfile
   for pkg in ("openproteo-io", "openproteo"):
       d = json.load(urllib.request.urlopen(f"https://pypi.org/pypi/{pkg}/json"))
       v = d["info"]["version"]; urls = d["releases"][v]
       sd = [u for u in urls if u["packagetype"] == "sdist"]
       raw = urllib.request.urlopen(sd[0]["url"]).read()
       lic = [n for n in tarfile.open(fileobj=io.BytesIO(raw)).getnames() if "licen" in n.lower()]
       print(pkg, v, "sdist" if sd else "NO SDIST", "LICENSE" if lic else "NO LICENSE")
   PY
   ```

10. **Update `versions.toml`** in the ops repo and commit
    (`versions: bump OpenProteo to X.Y.Z`).

## conda-forge (two-stage)

11. The facade's recipe depends on the binding, so submit in order:
    - First `openproteo-io` to `conda-forge/staged-recipes`. Wait for the
      feedstock to be created and the package to appear on the channel.
    - Then `openproteo` (a `noarch: python` recipe whose `run` requirement
      includes `openproteo-io`).
    - Build each recipe locally in `condaforge/linux-anvil-cos7-x86_64`
      (install `conda-forge-pinning`, build with its
      `conda_build_config.yaml`) before opening the PR. Every recipe using a
      compiler must declare `{{ stdlib("c") }}` or the linter fails.
