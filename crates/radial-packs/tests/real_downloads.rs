//! Downloads every artifact in the shipped manifest for real and checks it:
//! the checksum matches, the archive unpacks, and the executable is where the
//! manifest says it is. Needs the network and ~250 MB, so it is opt-in:
//!
//!     cargo test --test real_downloads -- --ignored --nocapture
//!
//! Run it whenever the manifest changes, and periodically to catch upstream
//! moving or deleting a pinned file.

use radial_packs::installer::{Action, Installer};
use radial_packs::manifest::Manifest;

fn check_platform(platform: &str, packs: &[&str]) {
    let home = tempfile::tempdir().unwrap();
    let installer = Installer::new(home.path().to_path_buf(), Manifest::embedded(), platform.to_string());
    let packs: Vec<String> = packs.iter().map(|p| p.to_string()).collect();

    let plan = installer.plan(&packs).unwrap();
    for item in &plan.items {
        assert!(
            matches!(item.action, Action::Download { .. }),
            "{platform}: {} should be a download, got {:?}",
            item.engine,
            item.action
        );
    }

    installer.install(&packs, &mut |_| {}).unwrap();
    for item in &plan.items {
        let exe = Installer::find_engine(home.path(), item.engine)
            .unwrap_or_else(|| panic!("{platform}: {} not recorded", item.engine));
        assert!(exe.is_file(), "{platform}: {} missing at {}", item.engine, exe.display());
        println!("{platform}: {} ok at {}", item.engine, exe.display());
    }
}

#[test]
#[ignore = "downloads ~550 MB"]
fn macos_arm64_all_packs() {
    check_platform("macos-aarch64", &["text", "audio", "documents"]);
}

#[test]
#[ignore = "downloads ~550 MB"]
fn macos_x86_64_all_packs() {
    check_platform("macos-x86_64", &["text", "audio", "documents"]);
}

/// Windows only: unpacking an .msi needs `msiexec`, which exists only there.
#[cfg(windows)]
#[test]
#[ignore = "downloads ~600 MB"]
fn windows_x86_64_all_packs() {
    check_platform("windows-x86_64", &["text", "audio", "documents"]);
}

/// Installs LibreOffice from a copy of the .dmg already on disk (the shipped
/// manifest's checksum still applies), so the slow part can be skipped:
///
///     RADIAL_TEST_LO_DMG=/path/to/LibreOffice_..._aarch64.dmg \
///       cargo test --test real_downloads libreoffice_from_local_dmg -- --ignored --nocapture
#[cfg(target_os = "macos")]
#[test]
#[ignore = "needs RADIAL_TEST_LO_DMG and mounts a disk image"]
fn libreoffice_from_local_dmg() {
    let Some(dmg) = std::env::var_os("RADIAL_TEST_LO_DMG") else {
        eprintln!("skipping: RADIAL_TEST_LO_DMG not set");
        return;
    };
    let platform = radial_packs::platform::key();
    let mut manifest = Manifest::embedded();
    let artifact = manifest
        .engines
        .get_mut("libreoffice")
        .and_then(|r| r.platforms.get_mut(&platform))
        .expect("libreoffice is pinned for this platform");
    artifact.url = url::Url::from_file_path(&dmg).unwrap().to_string();

    let home = tempfile::tempdir().unwrap();
    let installer = Installer::new(home.path().to_path_buf(), manifest, platform);
    installer.install(&["documents".to_string()], &mut |_| {}).unwrap();

    let exe = Installer::find_engine(home.path(), "libreoffice").expect("recorded");
    println!("soffice at {}", exe.display());
    let lines = installer.doctor().unwrap();
    println!("doctor: {:?}", lines);
    assert!(lines[0].ok, "{:?}", lines[0]);
    assert!(lines[0].detail.contains("LibreOffice"), "{:?}", lines[0]);
}
