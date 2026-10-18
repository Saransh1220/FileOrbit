use std::process::{Command, Stdio};

/// Runs a converter to completion. On failure the error is the tail of stderr,
/// which is what the user needs to see ("no such codec", "file is corrupt").
pub fn run(mut cmd: Command) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: no console flashing on screen per conversion.
        cmd.creation_flags(0x0800_0000);
    }
    let output = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("could not start {:?}: {e}", cmd.get_program()))?;
    if output.status.success() {
        return Ok(());
    }
    Err(stderr_tail(&output.stderr))
}

fn stderr_tail(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let tail = lines[lines.len().saturating_sub(3)..].join(" ");
    if tail.is_empty() {
        "the converter failed without a message".to_string()
    } else {
        tail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_keeps_last_three_nonblank_lines() {
        assert_eq!(stderr_tail(b"a\n\nb\nc\nd\n"), "b c d");
    }

    #[test]
    fn empty_stderr_gets_a_message() {
        assert!(!stderr_tail(b"").is_empty());
    }
}
