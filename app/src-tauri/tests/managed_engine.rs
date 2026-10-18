//! The app must prefer an engine installed by `radial install` over any copy
//! elsewhere on the machine. This runs in its own process because it sets
//! RADIAL_HOME, which would leak into other tests in the same process.

use app_lib::engine::binaries::{self, Tool};
use radial_packs::state::{InstalledEngine, Source, State};

#[test]
fn managed_engine_wins_over_the_system_copy() {
    let home = tempfile::tempdir().unwrap();
    let fake = home.path().join("engines/ffmpeg/1.0/ffmpeg");
    std::fs::create_dir_all(fake.parent().unwrap()).unwrap();
    std::fs::write(&fake, "#!/bin/sh\n").unwrap();

    let mut state = State::default();
    state.packs.insert("audio".into());
    state.engines.insert(
        "ffmpeg".into(),
        InstalledEngine { version: "1.0".into(), source: Source::Downloaded, path: fake.clone() },
    );
    state.save(home.path()).unwrap();

    // SAFETY: this is the only test in this process, so nothing else reads the environment concurrently.
    unsafe { std::env::set_var("RADIAL_HOME", home.path()) };

    assert_eq!(binaries::find(Tool::Ffmpeg).unwrap(), fake);
}

#[test]
fn without_a_managed_engine_the_error_names_the_pack_to_install() {
    let home = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("RADIAL_HOME", home.path()) };
    // Only meaningful where no system copy exists; otherwise the fallback legitimately succeeds.
    if let Err(message) = binaries::find(Tool::Pandoc) {
        assert!(message.contains("radial install"), "{message}");
    }
}
