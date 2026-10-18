//! What the installed converters can actually do.
//!
//! ffmpeg builds differ (a Homebrew build may lack libwebp or libvorbis), so a
//! wedge is only offered when the local ffmpeg has the encoder it needs.

use super::binaries::{self, Tool};
use super::target::Engine;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Encoders of the ffmpeg at this path. Keyed by path so that installing a
/// pack (which changes which ffmpeg is used) is noticed without a restart.
type EncoderCache = Option<(PathBuf, Option<Arc<HashSet<String>>>)>;
static FFMPEG_ENCODERS: Mutex<EncoderCache> = Mutex::new(None);

/// Encoder an ffmpeg target depends on, where the default pick is optional.
fn required_encoder(target_id: &str) -> Option<&'static str> {
    match target_id {
        "webp" => Some("libwebp"),
        "ogg" => Some("libvorbis"),
        "mp3" => Some("libmp3lame"),
        _ => None,
    }
}

/// Probes ffmpeg once, off the UI thread, so the first ring opens instantly.
pub fn warm_up() {
    std::thread::spawn(|| {
        let _ = ffmpeg_encoders();
    });
}

fn ffmpeg_encoders() -> Option<Arc<HashSet<String>>> {
    let exe = binaries::find(Tool::Ffmpeg).ok()?;
    let mut cache = FFMPEG_ENCODERS.lock().unwrap();
    if let Some((path, encoders)) = cache.as_ref() {
        if *path == exe {
            return encoders.clone();
        }
    }
    let probed = Command::new(&exe)
        .args(["-hide_banner", "-encoders"])
        .output()
        .ok()
        .map(|o| Arc::new(parse_encoders(&String::from_utf8_lossy(&o.stdout))));
    *cache = Some((exe, probed.clone()));
    probed
}

/// Parses `ffmpeg -encoders`: rows look like ` A....D libmp3lame  description`.
fn parse_encoders(listing: &str) -> HashSet<String> {
    listing
        .lines()
        .skip_while(|l| !l.trim_start().starts_with("------"))
        .skip(1)
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let flags = parts.next()?;
            (flags.len() == 6).then(|| parts.next().map(str::to_string))?
        })
        .collect()
}

pub fn is_supported(engine: Engine, target_id: &str) -> bool {
    let Some(needed) = required_encoder(target_id).filter(|_| engine == Engine::Ffmpeg) else {
        return true;
    };
    // If ffmpeg is missing the conversion fails later with an install hint.
    ffmpeg_encoders().is_none_or(|set| set.contains(needed))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = "Encoders:\n V..... = Video\n ------\n V....D libx264              H.264\n A....D libmp3lame           MP3\n";

    #[test]
    fn parses_encoder_names_after_the_separator() {
        let set = parse_encoders(LISTING);
        assert!(set.contains("libmp3lame") && set.contains("libx264"));
        assert!(!set.contains("Video"));
    }

    #[test]
    fn targets_without_special_encoders_are_always_supported() {
        assert!(is_supported(Engine::Pandoc, "docx"));
        assert!(is_supported(Engine::Ffmpeg, "wav"));
    }
}
