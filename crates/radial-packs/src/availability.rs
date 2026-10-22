//! Which packs are usable right now, and where each program comes from.
//!
//! A program can be one we installed (recorded in `installed.json`) or one the
//! user already has on the computer. Both make a pack usable: the app finds and
//! uses either, so what the manager and `radial list` show must agree with it.

use crate::catalog::{self, PackDef};
use crate::state::{Source, State};
use std::path::PathBuf;

/// Where one program of a pack comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineStatus {
    pub id: &'static str,
    /// `None` when the program is not available at all.
    pub source: Option<Source>,
    /// Version of a program we installed; unknown for a copy already on the computer.
    pub version: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackAvailability {
    pub engines: Vec<EngineStatus>,
    /// Usable now: we installed it, or every program it needs is already on the computer.
    pub installed: bool,
    /// We installed it (so we can also remove it). False for a pack that is only
    /// usable thanks to programs the user already had.
    pub managed: bool,
}

/// `system` looks a program up on the computer by executable name (see `system::find`);
/// it is injected so the logic can be tested without touching the machine.
pub fn pack_availability(
    pack: &PackDef,
    state: &State,
    system: &dyn Fn(&str) -> Option<PathBuf>,
) -> PackAvailability {
    let engines: Vec<EngineStatus> = pack
        .engines
        .iter()
        .map(|id| {
            if let Some(ours) = state.engines.get(*id) {
                return EngineStatus {
                    id,
                    source: Some(ours.source),
                    version: Some(ours.version.clone()),
                    path: Some(ours.path.clone()),
                };
            }
            let found = catalog::engine(id).and_then(|def| system(def.exe));
            EngineStatus { id, source: found.as_ref().map(|_| Source::System), version: None, path: found }
        })
        .collect();

    let managed = state.packs.contains(pack.id);
    let all_available = engines.iter().all(|e| e.source.is_some());
    PackAvailability { installed: managed || all_available, managed, engines }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::InstalledEngine;

    fn nothing(_: &str) -> Option<PathBuf> {
        None
    }

    fn everything(exe: &str) -> Option<PathBuf> {
        Some(PathBuf::from(format!("/usr/bin/{exe}")))
    }

    fn pack(id: &str) -> &'static PackDef {
        catalog::pack(id).unwrap()
    }

    #[test]
    fn nothing_anywhere_means_not_installed() {
        let a = pack_availability(pack("documents"), &State::default(), &nothing);
        assert!(!a.installed && !a.managed);
        assert_eq!(a.engines[0].source, None);
    }

    #[test]
    fn a_program_already_on_the_computer_makes_the_pack_usable_but_not_removable() {
        let a = pack_availability(pack("documents"), &State::default(), &everything);
        assert!(a.installed, "the app uses this copy, so the manager must say so");
        assert!(!a.managed, "we did not install it, so there is nothing for us to remove");
        assert_eq!(a.engines[0].source, Some(Source::System));
        assert_eq!(a.engines[0].path, Some(PathBuf::from("/usr/bin/soffice")));
        assert_eq!(a.engines[0].version, None);
    }

    #[test]
    fn our_own_install_is_managed_and_reports_its_version() {
        let mut state = State::default();
        state.packs.insert("audio".into());
        state.engines.insert(
            "ffmpeg".into(),
            InstalledEngine { version: "9.0.2".into(), source: Source::Downloaded, path: "/x/ffmpeg".into() },
        );
        let a = pack_availability(pack("audio"), &state, &nothing);
        assert!(a.installed && a.managed);
        assert_eq!(a.engines[0].source, Some(Source::Downloaded));
        assert_eq!(a.engines[0].version.as_deref(), Some("9.0.2"));
    }

    #[test]
    fn our_copy_wins_over_the_one_on_the_computer() {
        let mut state = State::default();
        state.packs.insert("audio".into());
        state.engines.insert(
            "ffmpeg".into(),
            InstalledEngine { version: "9.0.2".into(), source: Source::Downloaded, path: "/ours/ffmpeg".into() },
        );
        let a = pack_availability(pack("audio"), &state, &everything);
        assert_eq!(a.engines[0].path, Some(PathBuf::from("/ours/ffmpeg")));
        assert_eq!(a.engines[0].source, Some(Source::Downloaded));
    }

    #[test]
    fn packs_sharing_a_program_are_all_usable_when_it_is_there() {
        for id in ["audio", "video", "images"] {
            assert!(pack_availability(pack(id), &State::default(), &everything).installed, "{id}");
        }
    }

    #[test]
    fn a_library_with_no_system_copy_stays_not_installed() {
        // PDFium is a library inside our own folder; nothing on PATH stands in for it.
        let only_ffmpeg = |exe: &str| (exe == "ffmpeg").then(|| PathBuf::from("/usr/bin/ffmpeg"));
        assert!(!pack_availability(pack("pdf"), &State::default(), &only_ffmpeg).installed);
        assert!(pack_availability(pack("audio"), &State::default(), &only_ffmpeg).installed);
    }
}
