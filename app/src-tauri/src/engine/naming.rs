use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Gives up after this many `name (N).ext` candidates.
const MAX_ATTEMPTS: u32 = 10_000;

/// `report.pdf`, then `report (1).pdf`, `report (2).pdf`, ...
fn candidate_name(stem: &str, ext: &str, attempt: u32) -> String {
    if attempt == 0 {
        format!("{stem}.{ext}")
    } else {
        format!("{stem} ({attempt}).{ext}")
    }
}

/// Picks the output path next to the source and **reserves it** by creating
/// an empty file there. Creating it atomically (`create_new`) is what keeps
/// two simultaneous jobs from choosing the same name, and it means an
/// existing file is never overwritten. The engine then fills the file in; if
/// the conversion fails, the caller must delete the reserved file.
pub fn reserve_output_path(source: &Path, ext: &str) -> Result<PathBuf, String> {
    reserve_with_suffix(source, "", ext)
}

/// Like `reserve_output_path`, with `suffix` added to the source's name first:
/// `report.pdf` + " (merged)" gives `report (merged).pdf`, then `report (merged) (1).pdf`.
pub fn reserve_with_suffix(source: &Path, suffix: &str, ext: &str) -> Result<PathBuf, String> {
    let dir = source.parent().unwrap_or_else(|| Path::new("."));
    let stem = format!(
        "{}{suffix}",
        source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    );

    for attempt in 0..MAX_ATTEMPTS {
        let candidate = dir.join(candidate_name(&stem, ext, attempt));
        match OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot create {}: {e}", candidate.display())),
        }
    }
    Err(format!("too many files named \"{stem}\" in that folder"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("radial-naming-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn names_are_numbered_after_the_first() {
        assert_eq!(candidate_name("report", "pdf", 0), "report.pdf");
        assert_eq!(candidate_name("report", "pdf", 2), "report (2).pdf");
    }

    #[test]
    fn repeated_conversions_never_overwrite() {
        let dir = scratch("repeat");
        let source = dir.join("report.docx");
        let first = reserve_output_path(&source, "pdf").unwrap();
        let second = reserve_output_path(&source, "pdf").unwrap();
        let third = reserve_output_path(&source, "pdf").unwrap();
        assert_eq!(first, dir.join("report.pdf"));
        assert_eq!(second, dir.join("report (1).pdf"));
        assert_eq!(third, dir.join("report (2).pdf"));
    }

    #[test]
    fn a_suffix_goes_after_the_name_and_is_still_numbered() {
        let dir = scratch("suffix");
        let source = dir.join("contract.pdf");
        let first = reserve_with_suffix(&source, " (merged)", "pdf").unwrap();
        let second = reserve_with_suffix(&source, " (merged)", "pdf").unwrap();
        assert_eq!(first, dir.join("contract (merged).pdf"));
        assert_eq!(second, dir.join("contract (merged) (1).pdf"));
    }

    #[test]
    fn an_existing_file_is_left_untouched() {
        let dir = scratch("existing");
        std::fs::write(dir.join("report.pdf"), b"precious").unwrap();
        let out = reserve_output_path(&dir.join("report.docx"), "pdf").unwrap();
        assert_eq!(out, dir.join("report (1).pdf"));
        assert_eq!(std::fs::read(dir.join("report.pdf")).unwrap(), b"precious");
    }

    #[test]
    fn names_with_dots_and_spaces_keep_their_stem() {
        let dir = scratch("dots");
        let out = reserve_output_path(&dir.join("Q3 v1.2 final.docx"), "pdf").unwrap();
        assert_eq!(out, dir.join("Q3 v1.2 final.pdf"));
    }

    #[test]
    fn unwritable_folder_reports_an_error() {
        let err = reserve_output_path(Path::new("/nonexistent-dir-xyz/a.docx"), "pdf").unwrap_err();
        assert!(err.contains("cannot create"), "{err}");
    }
}
