use crate::engine::binaries::{self, Tool};
use crate::engine::process;
use std::path::Path;
use std::process::Command;

pub fn convert(input: &Path, output: &Path, to_ext: &str) -> Result<(), String> {
    let format = if to_ext == "jpg" { "jpeg" } else { to_ext };
    let mut cmd = Command::new(binaries::find(Tool::Sips)?);
    cmd.args(["-s", "format", format]).arg(input).arg("--out").arg(output);
    process::run(cmd)
}
