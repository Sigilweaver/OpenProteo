//! `vendor2mzml`: detect the vendor format of an input path and write
//! mzML to an output path via the appropriate parser.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: vendor2mzml <input.raw | bundle.d | bundle.raw> <output.mzML> [--indexed]");
        return ExitCode::from(2);
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let indexed = args.iter().skip(3).any(|a| a == "--indexed");

    let Some(detected) = openproteo_io::detect_format(&input) else {
        eprintln!(
            "error: {} does not look like a supported vendor format",
            input.display()
        );
        eprintln!("  (recognized: Thermo .raw, Bruker .d/, Waters .raw/)");
        return ExitCode::from(1);
    };
    eprintln!("detected: {} <- {}", detected.format.name(), input.display());

    let t0 = Instant::now();
    if let Err(e) = openproteo_io::convert_to_mzml(detected, &output, indexed) {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    let tag = if indexed { " (indexed)" } else { "" };
    eprintln!(
        "wrote {} in {:.1}s{tag}",
        output.display(),
        t0.elapsed().as_secs_f64()
    );
    ExitCode::SUCCESS
}
