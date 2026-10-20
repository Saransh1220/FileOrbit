use crate::engine::binaries::{self, Tool};
use crate::engine::process;
use std::path::Path;
use std::process::Command;

fn reader(ext: &str) -> &str {
    match ext {
        "md" | "txt" => "markdown",
        "tex" => "latex",
        other => other,
    }
}

fn writer(ext: &str) -> &str {
    match ext {
        "md" => "gfm",
        "txt" => "plain",
        other => other,
    }
}

pub fn convert(input: &Path, from_ext: &str, output: &Path, to_ext: &str) -> Result<(), String> {
    let mut cmd = Command::new(binaries::find(Tool::Pandoc)?);
    cmd.args(["-f", reader(from_ext), "-t", writer(to_ext)])
        .arg(input)
        .arg("-o")
        .arg(output);
    // Binary writers pull embedded media out of the source into the output.
    if from_ext == "docx" || from_ext == "odt" {
        cmd.arg("--extract-media").arg(output.with_extension("media"));
    }
    process::run(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_names_map_to_pandoc_names() {
        assert_eq!(reader("md"), "markdown");
        assert_eq!(reader("tex"), "latex");
        assert_eq!(writer("txt"), "plain");
        assert_eq!(writer("docx"), "docx");
    }
}
