use crate::engine::binaries::{self, Tool};
use crate::engine::process;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// One LibreOffice conversion at a time. They share a settings profile that
/// LibreOffice locks while it runs, so two at once would block or fail.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// LibreOffice names its own output (`<stem>.<ext>` in --outdir), so it
/// converts into a private scratch directory and the result is copied to the
/// requested path.
///
/// The settings profile is kept between conversions. Making a new one costs
/// 10 to 20 seconds on every run (measured), against about 5 seconds when it is reused.
pub fn convert(input: &Path, output: &Path, to_ext: &str, job_id: u64) -> Result<(), String> {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let work = std::env::temp_dir().join(format!("radial-convert-{}-{job_id}", std::process::id()));
    std::fs::create_dir_all(&work).map_err(|e| format!("cannot create scratch dir: {e}"))?;
    let profile = profile_dir();

    let mut result = run_in(&work, &profile, input, output, to_ext);
    if result.is_err() {
        // A crashed run can leave the profile damaged; one retry on a clean one is cheap insurance.
        let _ = std::fs::remove_dir_all(&profile);
        result = run_in(&work, &profile, input, output, to_ext);
    }
    let _ = std::fs::remove_dir_all(&work);
    result
}

/// Where the kept profile lives: inside the app's data folder, next to the converters.
fn profile_dir() -> PathBuf {
    radial_packs::paths::default_home().join("libreoffice-profile")
}

fn run_in(work: &Path, profile: &Path, input: &Path, output: &Path, to_ext: &str) -> Result<(), String> {
    std::fs::create_dir_all(profile).map_err(|e| format!("cannot create the LibreOffice profile: {e}"))?;
    let profile_url = url::Url::from_directory_path(profile).map_err(|_| "profile path is not a valid URL".to_string())?;
    let mut cmd = Command::new(binaries::find(Tool::LibreOffice)?);
    cmd.arg(format!("-env:UserInstallation={profile_url}"))
        .args(["--headless", "--convert-to", filter(to_ext), "--outdir"])
        .arg(work)
        .arg(input);
    process::run(cmd)?;

    let produced = produced_path(work, input, to_ext);
    if !produced.is_file() {
        return Err("LibreOffice finished but produced no file".to_string());
    }
    std::fs::copy(&produced, output).map_err(|e| format!("cannot write output: {e}"))?;
    Ok(())
}

/// `--convert-to` value; plain-text export needs an explicit filter name.
fn filter(to_ext: &str) -> &str {
    match to_ext {
        "txt" => "txt:Text",
        other => other,
    }
}

fn produced_path(work: &Path, input: &Path, to_ext: &str) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default();
    work.join(stem).with_extension(to_ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn txt_uses_the_text_filter() {
        assert_eq!(filter("txt"), "txt:Text");
        assert_eq!(filter("pdf"), "pdf");
    }

    #[test]
    fn produced_file_sits_in_scratch_dir_with_new_extension() {
        let p = produced_path(Path::new("/w"), Path::new("/docs/My Report.docx"), "pdf");
        assert_eq!(p, PathBuf::from("/w/My Report.pdf"));
    }
}
