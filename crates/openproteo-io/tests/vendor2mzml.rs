//! End-to-end smoke test: detect each vendor format and write a tiny
//! mzML to a tempfile, asserting it is non-empty and starts with the
//! mzML preamble. Each branch is skipped silently when the
//! corresponding corpus path is not present so the test stays green on
//! a vanilla checkout.

use std::fs;
use std::path::PathBuf;

fn smoke(input: PathBuf) {
    if !input.exists() {
        eprintln!("skipping {}: corpus not present", input.display());
        return;
    }
    let det = openproteo_io::detect_format(&input).expect("detect");
    let out = std::env::temp_dir().join(format!(
        "msio-smoke-{}-{}.mzML",
        det.format.name(),
        std::process::id()
    ));
    openproteo_io::convert_to_mzml(det, &out, false).expect("convert");
    let bytes = fs::read(&out).expect("read");
    assert!(
        bytes.len() > 4096,
        "mzML suspiciously small: {}",
        bytes.len()
    );
    let head = std::str::from_utf8(&bytes[..256.min(bytes.len())]).unwrap_or("");
    assert!(head.contains("<?xml"), "missing xml preamble");
    assert!(
        head.contains("mzML") || bytes.windows(4).any(|w| w == b"mzML"),
        "missing mzML root tag"
    );
    let _ = fs::remove_file(&out);
}

#[test]
fn thermo_smoke() {
    smoke(PathBuf::from(
        "../../../ProLance/corpus/thermo/PXD068962_Q_Exactive_UHMR_insource-CID.raw",
    ));
}

#[test]
fn waters_smoke() {
    smoke(PathBuf::from(
        "../../../ProLance/corpus/waters/PXD058812/molecular_mass_P15_01.raw",
    ));
}

#[test]
fn bruker_smoke() {
    smoke(PathBuf::from(
        "../../../OpenTimsTDF/re/artifacts/cache/pride/PXD036417/NQO1-F107C_coi-N2-P_200-0C_3996.d",
    ));
}
