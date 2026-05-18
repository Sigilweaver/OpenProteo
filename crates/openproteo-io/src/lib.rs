//! `openproteo-io` is the umbrella crate that ties together the open
//! Rust mass-spec parsers (`opentfraw`, `opentimstdf`, `openwraw`)
//! behind a uniform vendor-detection + mzML-conversion API.
//!
//! Each vendor parser is gated behind a Cargo feature
//! (`thermo`, `bruker`, `waters`) and re-exported under
//! [`vendor`]. The `all` meta-feature pulls in every supported
//! vendor.
//!
//! Even with no features enabled, [`detect_format`] is available so
//! callers can probe a path without paying the compile-time cost of a
//! parser they will not use.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

pub use openproteo_core as core;

#[cfg(feature = "arrow")]
pub use openproteo_core::arrow as arrow;

/// Re-exports of each vendor parser, gated by feature.
pub mod vendor {
    #[cfg(feature = "thermo")]
    pub use opentfraw;
    #[cfg(feature = "bruker")]
    pub use opentimstdf;
    #[cfg(feature = "waters")]
    pub use openwraw;
}

/// Detected on-disk vendor / format family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorFormat {
    /// Thermo Fisher Finnigan `.raw` (file).
    ThermoRaw,
    /// Bruker timsTOF TDF (directory ending in `.d/` containing
    /// `analysis.tdf` + `analysis.tdf_bin`).
    BrukerTdf,
    /// Waters MassLynx bundle (directory ending in `.raw/` containing
    /// `_HEADER.TXT`).
    WatersRaw,
}

impl VendorFormat {
    /// Vendor-name string suitable for logs and the CLI.
    pub fn name(self) -> &'static str {
        match self {
            Self::ThermoRaw => "thermo",
            Self::BrukerTdf => "bruker",
            Self::WatersRaw => "waters",
        }
    }
}

/// Result of probing a filesystem path for a supported vendor format.
#[derive(Debug, Clone)]
pub struct Detected {
    /// Canonical path to feed back into the matching vendor reader.
    /// For directory-based formats this is the bundle directory; for
    /// Thermo, the `.raw` file itself.
    pub path: PathBuf,
    /// Identified format.
    pub format: VendorFormat,
}

/// Inspect `path` (file or directory) and return the matching vendor
/// format, or `None` when none of the supported signatures match.
///
/// This function is always available, even with no features enabled,
/// so a host application can decide which feature to enable at compile
/// time based on a runtime probe.
pub fn detect_format(path: &Path) -> Option<Detected> {
    if path.is_dir() {
        // Bruker .d/ first, then Waters .raw/.
        if path.join("analysis.tdf").is_file() && path.join("analysis.tdf_bin").is_file() {
            return Some(Detected {
                path: path.to_path_buf(),
                format: VendorFormat::BrukerTdf,
            });
        }
        if path.join("_HEADER.TXT").is_file() {
            return Some(Detected {
                path: path.to_path_buf(),
                format: VendorFormat::WatersRaw,
            });
        }
        return None;
    }
    if path.is_file() {
        if is_thermo_raw(path) {
            return Some(Detected {
                path: path.to_path_buf(),
                format: VendorFormat::ThermoRaw,
            });
        }
        return None;
    }
    None
}

/// Returns `true` if the file looks like a Thermo Finnigan `.raw`.
///
/// The Finnigan signature is the UTF-16LE string `Finnigan` starting at
/// offset 2 (the first two bytes are a small header version word).
fn is_thermo_raw(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 18];
    if f.read_exact(&mut buf).is_err() {
        return false;
    }
    // "Finnigan" in UTF-16LE: F.i.n.n.i.g.a.n. (16 bytes) at offset 2.
    const FINNIGAN_UTF16LE: [u8; 16] = [
        0x46, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x6e, 0x00, 0x69, 0x00, 0x67, 0x00, 0x61, 0x00, 0x6e,
        0x00,
    ];
    buf[2..18] == FINNIGAN_UTF16LE
}

/// Convert a detected vendor file to mzML at `output`. Picks the
/// correct vendor crate's `write_mzml` (or `write_indexed_mzml`) based
/// on `indexed`. Errors are returned as boxed `dyn Error` so callers do
/// not need to thread vendor-specific error types.
#[allow(clippy::needless_pass_by_value)] // for symmetry with detect_format
pub fn convert_to_mzml(
    detected: Detected,
    output: &Path,
    indexed: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use std::io::BufWriter;
    let f = File::create(output)?;
    let mut w = BufWriter::new(f);
    write_to(detected.format, &detected.path, &mut w, indexed)
}

fn write_to(
    format: VendorFormat,
    path: &Path,
    w: &mut impl std::io::Write,
    indexed: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match format {
        VendorFormat::ThermoRaw => {
            #[cfg(feature = "thermo")]
            {
                thermo_convert(path, w, indexed)
            }
            #[cfg(not(feature = "thermo"))]
            {
                let _ = (path, w, indexed);
                Err("openproteo-io was built without the 'thermo' feature".into())
            }
        }
        VendorFormat::BrukerTdf => {
            #[cfg(feature = "bruker")]
            {
                if indexed {
                    opentimstdf::mzml::write_indexed_mzml(path, w)?;
                } else {
                    opentimstdf::mzml::write_mzml(path, w)?;
                }
                Ok(())
            }
            #[cfg(not(feature = "bruker"))]
            {
                let _ = (path, w, indexed);
                Err("openproteo-io was built without the 'bruker' feature".into())
            }
        }
        VendorFormat::WatersRaw => {
            #[cfg(feature = "waters")]
            {
                if indexed {
                    openwraw::mzml::write_indexed_mzml(path, w)?;
                } else {
                    openwraw::mzml::write_mzml(path, w)?;
                }
                Ok(())
            }
            #[cfg(not(feature = "waters"))]
            {
                let _ = (path, w, indexed);
                Err("openproteo-io was built without the 'waters' feature".into())
            }
        }
    }
}

#[cfg(feature = "thermo")]
fn thermo_convert(
    path: &Path,
    out: &mut impl std::io::Write,
    indexed: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use std::io::BufReader;
    let raw = opentfraw::RawFileReader::open_path(path)?;
    let mut source = BufReader::with_capacity(2 << 20, File::open(path)?);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.raw");
    if indexed {
        opentfraw::mzml::write_indexed_mzml(&raw, &mut source, out, filename, false)?;
    } else {
        opentfraw::mzml::write_mzml(&raw, &mut source, out, filename, false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detect_returns_none_for_garbage_file() {
        let tmp = tempfile_path();
        std::fs::write(&tmp, b"hello").unwrap();
        assert!(detect_format(&tmp).is_none());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn detect_returns_thermo_for_finnigan_magic() {
        let tmp = tempfile_path();
        let mut f = std::fs::File::create(&tmp).unwrap();
        // 2-byte version word + "Finnigan" in UTF-16LE + trailing garbage.
        f.write_all(&[
            0x01, 0xa1, 0x46, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x6e, 0x00, 0x69, 0x00, 0x67, 0x00,
            0x61, 0x00, 0x6e, 0x00, 0xff, 0xff,
        ])
        .unwrap();
        let det = detect_format(&tmp).expect("detect");
        assert_eq!(det.format, VendorFormat::ThermoRaw);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn detect_returns_bruker_for_tdf_layout() {
        let tmp = tempfile_dir();
        std::fs::write(tmp.join("analysis.tdf"), b"").unwrap();
        std::fs::write(tmp.join("analysis.tdf_bin"), b"").unwrap();
        let det = detect_format(&tmp).expect("detect");
        assert_eq!(det.format, VendorFormat::BrukerTdf);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn detect_returns_waters_for_header_layout() {
        let tmp = tempfile_dir();
        std::fs::write(tmp.join("_HEADER.TXT"), b"$$ FAKE\n").unwrap();
        let det = detect_format(&tmp).expect("detect");
        assert_eq!(det.format, VendorFormat::WatersRaw);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn tempfile_path() -> PathBuf {
        let pid = std::process::id();
        let mut p = std::env::temp_dir();
        p.push(format!("msio-test-{pid}-{:p}", &pid));
        p
    }

    fn tempfile_dir() -> PathBuf {
        let p = tempfile_path();
        let _ = std::fs::create_dir_all(&p);
        p
    }
}
