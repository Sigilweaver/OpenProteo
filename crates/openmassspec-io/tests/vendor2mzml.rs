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
    let det = openmassspec_io::detect_format(&input).expect("detect");
    let out = std::env::temp_dir().join(format!(
        "msio-smoke-{}-{}.mzML",
        det.format.name(),
        std::process::id()
    ));
    openmassspec_io::convert_to_mzml(det, &out, false).expect("convert");
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
        "../../../SpecLance/corpus/thermo/PXD068962_Q_Exactive_UHMR_insource-CID.raw",
    ));
}

#[test]
fn waters_smoke() {
    smoke(PathBuf::from(
        "../../../SpecLance/corpus/waters/PXD058812/molecular_mass_P15_01.raw",
    ));
}

#[test]
fn bruker_smoke() {
    smoke(PathBuf::from(
        "../../../OpenTimsTDF/re/artifacts/cache/pride/PXD036417/NQO1-F107C_coi-N2-P_200-0C_3996.d",
    ));
}

/// After `convert_to_mzml_centroided`, no spectrum in the output should
/// still be tagged profile mode - every profile spectrum was centroided,
/// and every already-centroid spectrum passed through unchanged. This
/// holds regardless of the input file's actual mode mix, so it's a
/// meaningful assertion even against real-world corpus data.
fn centroid_smoke(input: PathBuf) {
    if !input.exists() {
        eprintln!("skipping {}: corpus not present", input.display());
        return;
    }
    let det = openmassspec_io::detect_format(&input).expect("detect");
    let out = std::env::temp_dir().join(format!(
        "msio-centroid-smoke-{}-{}.mzML",
        det.format.name(),
        std::process::id()
    ));
    openmassspec_io::convert_to_mzml_centroided(det, &out, false, None).expect("convert");
    let text = fs::read_to_string(&out).expect("read");
    assert!(
        !text.contains(r#"accession="MS:1000128""#),
        "output still contains a profile spectrum cvParam after centroiding"
    );
    assert!(
        text.contains(r#"accession="MS:1000127""#),
        "output has no centroid spectrum cvParam at all"
    );
    let _ = fs::remove_file(&out);
}

#[test]
fn thermo_centroid_smoke() {
    centroid_smoke(PathBuf::from(
        "../../../SpecLance/corpus/thermo/PXD068962_Q_Exactive_UHMR_insource-CID.raw",
    ));
}

#[test]
fn waters_centroid_smoke() {
    centroid_smoke(PathBuf::from(
        "../../../SpecLance/corpus/waters/PXD058812/molecular_mass_P15_01.raw",
    ));
}

#[test]
fn bruker_centroid_smoke() {
    centroid_smoke(PathBuf::from(
        "../../../OpenTimsTDF/re/artifacts/cache/pride/PXD036417/NQO1-F107C_coi-N2-P_200-0C_3996.d",
    ));
}

/// `stream()`/`metadata_only()` must agree with `collect()` on both the
/// records visited and the run metadata returned, for every vendor. This
/// is the property issue #3 asked for: a lazy path that doesn't buffer the
/// whole run into a `Vec`, without silently diverging from the existing
/// two-pass API.
fn stream_matches_collect(input: PathBuf) {
    if !input.exists() {
        eprintln!("skipping {}: corpus not present", input.display());
        return;
    }
    let det = openmassspec_io::detect_format(&input).expect("detect");

    let (collected, collect_meta) = openmassspec_io::collect(det.clone()).expect("collect");

    let mut streamed = Vec::new();
    let stream_meta = openmassspec_io::stream(det.clone(), |rec| {
        streamed.push(rec);
        Ok(())
    })
    .expect("stream");

    assert_eq!(streamed.len(), collected.len(), "record count mismatch");
    for (a, b) in streamed.iter().zip(collected.iter()) {
        assert_eq!(a.native_id, b.native_id);
        assert_eq!(a.index, b.index);
        assert_eq!(a.mz, b.mz);
        assert_eq!(a.intensity, b.intensity);
    }

    let meta_only = openmassspec_io::metadata_only(det).expect("metadata_only");
    assert_eq!(meta_only.instrument.name, collect_meta.instrument.name);
    assert_eq!(meta_only.source_file_name, collect_meta.source_file_name);
    assert_eq!(stream_meta.instrument.name, collect_meta.instrument.name);
}

#[test]
fn thermo_stream_matches_collect() {
    stream_matches_collect(PathBuf::from(
        "../../../SpecLance/corpus/thermo/PXD068962_Q_Exactive_UHMR_insource-CID.raw",
    ));
}

#[test]
fn waters_stream_matches_collect() {
    stream_matches_collect(PathBuf::from(
        "../../../SpecLance/corpus/waters/PXD058812/molecular_mass_P15_01.raw",
    ));
}

#[test]
fn bruker_stream_matches_collect() {
    stream_matches_collect(PathBuf::from(
        "../../../OpenTimsTDF/re/artifacts/cache/pride/PXD036417/NQO1-F107C_coi-N2-P_200-0C_3996.d",
    ));
}
