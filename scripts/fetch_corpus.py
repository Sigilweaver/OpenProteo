"""
Shared corpus fetcher for the OpenMassSpec stack.

Generalized port of OpenTFRaw's per-repo fetcher. Reads a vendor-
agnostic ``sources.json``, resolves file URLs through the PRIDE REST
API (with FTP fallback), downloads files into a repo-local corpus
directory, and writes a ``manifest.json`` keyed by
``{accession}/{original_filename}``.

Schema: each `sources.json` is a list of vendor-tagged entries with
``url``, ``sha256``, ``ext`` (file extension), ``vendor``, ``mode``
(acquisition mode), and an optional ``accession`` (e.g. PRIDE/MetaboLights).

Per-repo wrappers pass repo-local paths in via the CLI; the script
itself contains no vendor- or repo-specific defaults beyond an
optional file-extension regex used to scan PRIDE FTP listings.

Usage::

    python fetch_corpus.py \\
        --sources path/to/sources.json \\
        --corpus-dir path/to/corpus \\
        [--manifest path/to/manifest.json] \\
        [--ext-pattern '\\.[Rr][Aa][Ww]$'] \\
        [--dry-run]

    python fetch_corpus.py --list-files ACCESSION \\
        [--ext-pattern '\\.[Rr][Aa][Ww]$']

Limitations: downloads single files only. Directory-bundle vendor
formats (Bruker ``.d/``, Waters ``.raw/``) need a recursive-fetch
mode that is not yet implemented.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
import urllib.request
from pathlib import Path

USER_AGENT = "OpenMassSpec-CorpusFetcher/1.0"
PRIDE_API = "https://www.ebi.ac.uk/pride/ws/archive/v2"
FTP_BASE = "https://ftp.pride.ebi.ac.uk/pride/data/archive"
DEFAULT_EXT_PATTERN = r"\.[Rr][Aa][Ww]$"


def _req(url: str) -> urllib.request.Request:
    return urllib.request.Request(url, headers={"User-Agent": USER_AGENT})


def project_pub_date(accession: str) -> str | None:
    try:
        with urllib.request.urlopen(
            _req(f"{PRIDE_API}/projects/{accession}"), timeout=20
        ) as r:
            return json.loads(r.read()).get("publicationDate")
    except Exception:
        return None


def ftp_dir_url(accession: str, pub_date: str) -> str:
    yyyy, mm = pub_date[:4], pub_date[5:7]
    return f"{FTP_BASE}/{yyyy}/{mm}/{accession}/"


def ftp_file_url(accession: str, filename: str, pub_date: str) -> str:
    return ftp_dir_url(accession, pub_date) + filename


def list_ftp_files(
    accession: str, pub_date: str, ext_regex: re.Pattern[str]
) -> list[str]:
    url = ftp_dir_url(accession, pub_date)
    try:
        with urllib.request.urlopen(_req(url), timeout=30) as r:
            html = r.read().decode("utf-8", errors="replace")
    except Exception as e:
        print(f"  [ERROR] FTP listing failed ({url}): {e}", flush=True)
        return []
    names = re.findall(r'<a\s+href="([^"/][^"]*)"', html)
    return sorted({n for n in names if ext_regex.search(n)})


def resolve_url(
    accession: str, filename: str, pub_date: str | None = None
) -> tuple[str, int] | None:
    page, page_size = 0, 100
    api_returned_data = False
    while True:
        api_url = (
            f"{PRIDE_API}/files/byProject"
            f"?accession={accession}&pageSize={page_size}&page={page}"
        )
        try:
            with urllib.request.urlopen(_req(api_url), timeout=30) as r:
                raw = r.read()
            if not raw:
                break
            data = json.loads(raw)
        except Exception as e:
            print(f"  [ERROR] PRIDE API: {e}", flush=True)
            break

        api_returned_data = True
        for entry in data.get("content", []):
            if entry.get("fileName", "").lower() == filename.lower():
                size = entry.get("fileSizeBytes", 0)
                for loc in entry.get("publicFileLocations", []):
                    val: str = loc.get("value", "")
                    if val.startswith("ftp://"):
                        return (
                            val.replace(
                                "ftp://ftp.pride.ebi.ac.uk",
                                "https://ftp.pride.ebi.ac.uk",
                                1,
                            ),
                            size,
                        )
                    if val.startswith("https://"):
                        return val, size
                return None

        pi = data.get("page", {})
        if (page + 1) * page_size >= pi.get("totalElements", 0):
            break
        page += 1
        time.sleep(0.2)

    if api_returned_data:
        return None

    print("  [INFO] API empty; using FTP fallback", flush=True)
    if pub_date is None:
        pub_date = project_pub_date(accession)
    if pub_date is None:
        print(f"  [ERROR] no pub date for {accession}", flush=True)
        return None
    return ftp_file_url(accession, filename, pub_date), 0


def load_manifest(path: Path) -> dict:
    if not path.exists():
        return {}
    with open(path) as f:
        return json.load(f)


def save_manifest(path: Path, manifest: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w") as f:
        json.dump(manifest, f, indent=2)
        f.write("\n")


def downloaded_for(manifest: dict, accession: str) -> list[str]:
    pfx = f"{accession}/"
    return [k[len(pfx):] for k in manifest if k.startswith(pfx)]


def _download_file(url: str, dest: Path) -> bool:
    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_suffix(dest.suffix + ".part")
    try:
        with urllib.request.urlopen(_req(url), timeout=600) as r, \
             open(tmp, "wb") as f:
            while chunk := r.read(1 << 20):
                f.write(chunk)
        tmp.rename(dest)
        return True
    except Exception as e:
        print(f"  [ERROR] download failed: {e}", flush=True)
        if tmp.exists():
            tmp.unlink()
        return False


def fetch_one(
    accession: str,
    filename: str,
    instrument: str,
    manifest: dict,
    manifest_path: Path,
    corpus_dir: Path,
    dry_run: bool,
    pub_date: str | None = None,
) -> None:
    result = resolve_url(accession, filename, pub_date)
    if result is None:
        print(f"  [WARN] could not resolve URL for {filename}", flush=True)
        return
    url, size = result
    size_str = f"{size / 1e6:.1f} MB" if size else "size unknown"
    lbl = instrument.replace(" ", "_")
    dest = corpus_dir / f"{accession}_{lbl}_{filename}"
    print(f"  {filename}  ({size_str})", flush=True)
    if dry_run:
        print(f"  [DRY-RUN] would write {dest.name}", flush=True)
        return
    if _download_file(url, dest):
        actual = dest.stat().st_size
        print(f"  Done: {actual / 1e6:.1f} MB", flush=True)
        manifest[f"{accession}/{filename}"] = {
            "instrument": instrument,
            "dest_filename": dest.name,
            "size_bytes": actual,
        }
        save_manifest(manifest_path, manifest)
    time.sleep(1)


def run(
    sources_path: Path,
    corpus_dir: Path,
    manifest_path: Path,
    ext_regex: re.Pattern[str],
    dry_run: bool,
) -> None:
    corpus_dir.mkdir(parents=True, exist_ok=True)
    with open(sources_path) as f:
        sources: list[dict] = json.load(f)

    manifest = load_manifest(manifest_path)

    for entry in sources:
        instrument: str = entry["instrument"]
        accession: str = entry["accession"]
        mode: str | None = entry.get("acquisition_mode") or entry.get("mode")
        explicit: list[str] = entry.get("files") or (
            [entry["pride_filename"]] if "pride_filename" in entry else []
        )
        count: int | None = entry.get("count")

        lbl_mode = f" ({mode})" if mode else ""
        print(f"\n{'=' * 60}", flush=True)
        print(f"  {instrument}{lbl_mode}  ({accession})", flush=True)

        already = downloaded_for(manifest, accession)

        for fname in explicit:
            if fname in already:
                print(f"  Already have: {fname}  -- skipping", flush=True)
                continue
            fetch_one(
                accession, fname, instrument, manifest, manifest_path,
                corpus_dir, dry_run,
            )
            already = downloaded_for(manifest, accession)

        if count is not None:
            need = count - len(already)
            if need <= 0:
                print(
                    f"  count={count} satisfied ({len(already)} files)  -- skipping",
                    flush=True,
                )
                continue
            pub_date = project_pub_date(accession)
            if not pub_date:
                print(f"  [ERROR] no pub date for {accession}", flush=True)
                continue
            available = list_ftp_files(accession, pub_date, ext_regex)
            if not available:
                print(f"  [WARN] FTP listing empty for {accession}", flush=True)
                continue
            candidates = [f for f in available if f not in already]
            if not candidates:
                print(
                    f"  All {len(available)} available files already downloaded.",
                    flush=True,
                )
                continue
            print(
                f"  Auto-fill: need {need} more, "
                f"{len(candidates)} candidates from {len(available)} total",
                flush=True,
            )
            for fname in candidates[:need]:
                fetch_one(
                    accession, fname, instrument, manifest, manifest_path,
                    corpus_dir, dry_run, pub_date,
                )
                already = downloaded_for(manifest, accession)

    print(f"\n{'=' * 60}", flush=True)
    files = sorted(p for p in corpus_dir.iterdir() if ext_regex.search(p.name))
    total = sum(f.stat().st_size for f in files)
    print(f"Corpus: {len(files)} file(s), {total / 1e9:.2f} GB total", flush=True)
    for f in files:
        print(f"  {f.name}  ({f.stat().st_size / 1e6:.1f} MB)", flush=True)


def cmd_list_files(accession: str, ext_regex: re.Pattern[str]) -> None:
    pub_date = project_pub_date(accession)
    if not pub_date:
        print(f"[ERROR] no publication date for {accession}")
        return
    files = list_ftp_files(accession, pub_date, ext_regex)
    print(f"{accession}  published {pub_date}  |  {len(files)} file(s)")
    print(f"  FTP dir: {ftp_dir_url(accession, pub_date)}")
    for fname in files:
        print(f"    {fname}")


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--sources", type=Path,
                   help="path to sources.json (required unless --list-files)")
    p.add_argument("--corpus-dir", type=Path,
                   help="target directory for downloads "
                        "(required unless --list-files or --dry-run)")
    p.add_argument("--manifest", type=Path,
                   help="manifest.json path (default: <corpus-dir>/manifest.json)")
    p.add_argument("--ext-pattern", default=DEFAULT_EXT_PATTERN,
                   help=f"regex matched against FTP filenames "
                        f"(default: {DEFAULT_EXT_PATTERN!r})")
    p.add_argument("--dry-run", action="store_true",
                   help="resolve URLs but do not download")
    p.add_argument("--list-files", metavar="ACCESSION",
                   help="list available files for a PRIDE project and exit")
    args = p.parse_args(argv)

    ext_regex = re.compile(args.ext_pattern)

    if args.list_files:
        cmd_list_files(args.list_files, ext_regex)
        return 0

    if args.sources is None:
        p.error("--sources is required")
    if args.corpus_dir is None:
        p.error("--corpus-dir is required")

    manifest_path = args.manifest or (args.corpus_dir / "manifest.json")
    run(args.sources, args.corpus_dir, manifest_path, ext_regex, args.dry_run)
    return 0


if __name__ == "__main__":
    sys.exit(main())
