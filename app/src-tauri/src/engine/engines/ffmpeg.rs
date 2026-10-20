use crate::engine::binaries::{self, Tool};
use crate::engine::process;
use std::path::Path;
use std::process::Command;

/// Audio, video and image conversion. ffmpeg picks codecs from the output
/// extension; the few flags below only improve defaults.
pub fn convert(input: &Path, output: &Path, to_ext: &str) -> Result<(), String> {
    let mut cmd = Command::new(binaries::find(Tool::Ffmpeg)?);
    cmd.args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(input);
    match to_ext {
        "jpg" => cmd.args(["-q:v", "2"]),
        "mp3" => cmd.args(["-vn", "-q:a", "2"]),
        "wav" | "flac" | "aac" | "m4a" | "ogg" => cmd.arg("-vn"),
        _ => &mut cmd,
    };
    cmd.arg(output);
    process::run(cmd)
}
