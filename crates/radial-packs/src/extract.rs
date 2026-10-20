//! Unpacking downloaded archives.

use crate::manifest::ArchiveKind;
use std::fs::File;
use std::path::Path;

/// Extracts `archive` into `dest`. `Raw` is not handled here (it is a plain copy).
pub fn extract(archive: &Path, kind: ArchiveKind, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| format!("cannot create {}: {e}", dest.display()))?;
    let file = File::open(archive).map_err(|e| format!("cannot open {}: {e}", archive.display()))?;
    match kind {
        ArchiveKind::Zip => zip::ZipArchive::new(file)
            .and_then(|mut z| z.extract(dest))
            .map_err(|e| format!("cannot unzip {}: {e}", archive.display())),
        ArchiveKind::TarGz => {
            let decoder = flate2::read::GzDecoder::new(file);
            tar::Archive::new(decoder)
                .unpack(dest)
                .map_err(|e| format!("cannot untar {}: {e}", archive.display()))
        }
        ArchiveKind::Raw => Err("raw files are copied, not extracted".to_string()),
        ArchiveKind::Dmg => {
            drop(file);
            dmg::extract(archive, dest)
        }
        ArchiveKind::Msi => {
            drop(file);
            msi::extract(archive, dest)
        }
    }
}

/// macOS disk images (LibreOffice): mount read-only, copy the app out, always unmount.
mod dmg {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Unmounts the image when dropped, so a failed copy never leaves a volume mounted.
    struct Mounted(PathBuf);

    impl Drop for Mounted {
        fn drop(&mut self) {
            let _ = Command::new("hdiutil").arg("detach").arg(&self.0).arg("-force").output();
            let _ = std::fs::remove_dir(&self.0);
        }
    }

    #[cfg(target_os = "macos")]
    pub fn extract(archive: &Path, dest: &Path) -> Result<(), String> {
        let mount = std::env::temp_dir().join(format!(
            "radial-mount-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&mount).map_err(|e| format!("cannot create mount point: {e}"))?;

        let attach = Command::new("hdiutil")
            .args(["attach", "-nobrowse", "-readonly", "-noverify", "-noautoopen", "-mountpoint"])
            .arg(&mount)
            .arg(archive)
            .output()
            .map_err(|e| format!("cannot run hdiutil: {e}"))?;
        if !attach.status.success() {
            let _ = std::fs::remove_dir(&mount);
            return Err(format!(
                "cannot open the disk image: {}",
                String::from_utf8_lossy(&attach.stderr).trim()
            ));
        }
        let mounted = Mounted(mount);

        let app = single_app(&mounted.0)?;
        // `ditto` keeps symlinks, permissions and code signatures intact, which a plain copy would not.
        let target = dest.join(app.file_name().unwrap_or_default());
        let copy = Command::new("ditto")
            .arg(&app)
            .arg(&target)
            .output()
            .map_err(|e| format!("cannot run ditto: {e}"))?;
        if !copy.status.success() {
            return Err(format!("cannot copy the app: {}", String::from_utf8_lossy(&copy.stderr).trim()));
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn extract(_archive: &Path, _dest: &Path) -> Result<(), String> {
        let _ = (&COUNTER, Ordering::Relaxed);
        Err("disk images (.dmg) can only be unpacked on macOS".to_string())
    }

    /// The one `*.app` at the top of the mounted volume.
    #[cfg(target_os = "macos")]
    fn single_app(volume: &Path) -> Result<PathBuf, String> {
        let apps: Vec<PathBuf> = std::fs::read_dir(volume)
            .map_err(|e| format!("cannot read the disk image: {e}"))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "app"))
            .collect();
        match apps.as_slice() {
            [one] => Ok(one.clone()),
            [] => Err("the disk image contains no application".to_string()),
            _ => Err("the disk image contains more than one application".to_string()),
        }
    }
}

/// Windows installer packages (LibreOffice): an administrative install unpacks
/// the files into a folder without installing or registering anything.
mod msi {
    use std::path::Path;

    #[cfg(windows)]
    pub fn extract(archive: &Path, dest: &Path) -> Result<(), String> {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        // msiexec wants `TARGETDIR="C:\path with spaces"`: the quotes go around the
        // value, not around the whole argument, so the command line is built by hand.
        // (The default install folder, "Radial Convert", has a space in it.)
        let quoted = |p: &Path| format!("\"{}\"", p.display());
        let mut log = dest.as_os_str().to_owned();
        log.push(".msi.log");
        let log = std::path::PathBuf::from(log);

        let output = Command::new("msiexec")
            .raw_arg("/a")
            .raw_arg(quoted(archive))
            .raw_arg("/qn")
            .raw_arg("/norestart")
            .raw_arg(format!("TARGETDIR={}", quoted(dest)))
            .raw_arg("/L*V")
            .raw_arg(quoted(&log))
            .output()
            .map_err(|e| format!("cannot run msiexec: {e}"))?;
        // 3010 means "a restart would finish the job", which an unpack never needs.
        match output.status.code() {
            Some(0) | Some(3010) => {
                let _ = std::fs::remove_file(&log);
                Ok(())
            }
            other => Err(format!(
                "the Windows installer service could not unpack the package (exit code {}). Details: {}",
                other.map_or("none".to_string(), |c| c.to_string()),
                log.display()
            )),
        }
    }

    #[cfg(not(windows))]
    pub fn extract(_archive: &Path, _dest: &Path) -> Result<(), String> {
        Err("installer packages (.msi) can only be unpacked on Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_zip(path: &Path) {
        let mut writer = zip::ZipWriter::new(File::create(path).unwrap());
        let options = zip::write::SimpleFileOptions::default().unix_permissions(0o755);
        writer.start_file("bin/tool", options).unwrap();
        writer.write_all(b"hello").unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn unzips_with_folders() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.zip");
        make_zip(&archive);
        let out = dir.path().join("out");
        extract(&archive, ArchiveKind::Zip, &out).unwrap();
        assert_eq!(std::fs::read(out.join("bin/tool")).unwrap(), b"hello");
    }

    #[test]
    fn untars_gz() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.tar.gz");
        let encoder = flate2::write::GzEncoder::new(File::create(&archive).unwrap(), flate2::Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        let data = b"payload";
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, "dir/file.txt", &data[..]).unwrap();
        builder.into_inner().unwrap().finish().unwrap();

        let out = dir.path().join("out");
        extract(&archive, ArchiveKind::TarGz, &out).unwrap();
        assert_eq!(std::fs::read(out.join("dir/file.txt")).unwrap(), b"payload");
    }

    #[test]
    fn a_garbage_disk_image_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("x.dmg");
        std::fs::write(&archive, "this is not a disk image").unwrap();
        assert!(extract(&archive, ArchiveKind::Dmg, &dir.path().join("o")).is_err());
    }

    #[cfg(not(windows))]
    #[test]
    fn msi_packages_say_they_need_windows() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("x.msi");
        std::fs::write(&archive, "x").unwrap();
        let err = extract(&archive, ArchiveKind::Msi, &dir.path().join("o")).unwrap_err();
        assert!(err.contains("Windows"), "{err}");
    }

}
