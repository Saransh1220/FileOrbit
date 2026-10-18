//! Real conversions through the same entry point the ring uses.
//! Each test skips (passes) when its converter is not installed.

use app_lib::engine::binaries::{self, Tool};
use app_lib::engine::capabilities;
use app_lib::engine::target::Engine;
use app_lib::engine::jobs::convert_file;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("radial-it-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn missing(tool: Tool) -> bool {
    let gone = binaries::find(tool).is_err();
    if gone {
        eprintln!("skipping: {tool:?} not installed");
    }
    gone
}

fn ffmpeg_make(args: &[&str], out: &Path) {
    let ffmpeg = binaries::find(Tool::Ffmpeg).unwrap();
    let status = Command::new(ffmpeg)
        .args(["-y", "-loglevel", "error"])
        .args(args)
        .arg(out)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn markdown_to_docx_and_back() {
    if missing(Tool::Pandoc) {
        return;
    }
    let dir = scratch("md");
    let md = dir.join("note.md");
    std::fs::write(&md, "# Title\n\nHello radial world.\n").unwrap();

    let docx = convert_file(&md, "docx", 1).unwrap();
    assert_eq!(docx, dir.join("note.docx"));
    assert!(std::fs::metadata(&docx).unwrap().len() > 0);

    let back = convert_file(&docx, "txt", 2);
    // docx -> txt is a LibreOffice route; only assert when it is installed.
    if binaries::find(Tool::LibreOffice).is_ok() {
        let text = std::fs::read_to_string(back.unwrap()).unwrap();
        assert!(text.contains("Hello radial world"));
    }
}

#[test]
fn audio_wav_mp3_roundtrip() {
    if missing(Tool::Ffmpeg) {
        return;
    }
    let dir = scratch("audio");
    let wav = dir.join("tone.wav");
    ffmpeg_make(&["-f", "lavfi", "-i", "sine=frequency=440:duration=1"], &wav);

    let mp3 = convert_file(&wav, "mp3", 3).unwrap();
    assert!(std::fs::metadata(&mp3).unwrap().len() > 1000);
    std::fs::remove_file(&wav).unwrap();
    let wav_again = convert_file(&mp3, "wav", 4).unwrap();
    assert!(std::fs::metadata(wav_again).unwrap().len() > 1000);
}

#[test]
fn image_png_to_jpg_and_webp() {
    if missing(Tool::Ffmpeg) {
        return;
    }
    let dir = scratch("image");
    let png = dir.join("pic.png");
    ffmpeg_make(&["-f", "lavfi", "-i", "color=c=blue:s=64x64", "-frames:v", "1"], &png);

    for target in ["jpg", "webp"] {
        if !capabilities::is_supported(Engine::Ffmpeg, target) {
            eprintln!("skipping {target}: ffmpeg lacks the encoder");
            continue;
        }
        let out = convert_file(&png, target, 5).unwrap();
        assert!(std::fs::metadata(out).unwrap().len() > 0, "{target}");
    }
}

#[test]
fn docx_to_pdf_via_libreoffice() {
    if missing(Tool::LibreOffice) || missing(Tool::Pandoc) {
        return;
    }
    let dir = scratch("pdf");
    let md = dir.join("report.md");
    std::fs::write(&md, "# Report\n\nBody text.\n").unwrap();
    let docx = convert_file(&md, "docx", 6).unwrap();

    let pdf = convert_file(&docx, "pdf", 7).unwrap();
    assert_eq!(pdf, dir.join("report.pdf"));
    let bytes = std::fs::read(pdf).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn unsupported_route_reports_a_clear_error() {
    let dir = scratch("bad");
    let file = dir.join("thing.xyz");
    std::fs::write(&file, "x").unwrap();
    let err = convert_file(&file, "pdf", 8).unwrap_err();
    assert!(err.contains("cannot convert"), "{err}");
}

#[test]
fn converting_twice_numbers_the_second_output() {
    if missing(Tool::Pandoc) {
        return;
    }
    let dir = scratch("twice");
    let md = dir.join("note.md");
    std::fs::write(&md, "# Title\n").unwrap();

    let first = convert_file(&md, "docx", 20).unwrap();
    let second = convert_file(&md, "docx", 21).unwrap();
    assert_eq!(first, dir.join("note.docx"));
    assert_eq!(second, dir.join("note (1).docx"));
    assert!(std::fs::metadata(&second).unwrap().len() > 0);
}

#[test]
fn failed_conversion_leaves_no_empty_file_behind() {
    if missing(Tool::Ffmpeg) {
        return;
    }
    let dir = scratch("failed");
    let fake = dir.join("broken.mp3");
    std::fs::write(&fake, "this is not audio").unwrap();

    assert!(convert_file(&fake, "wav", 22).is_err());
    assert!(!dir.join("broken.wav").exists(), "reserved output must be removed");
}

/// Needs only LibreOffice, so it can prove which LibreOffice the app uses
/// (run it with RADIAL_NO_SYSTEM_TOOLS=1 and RADIAL_HOME pointing at a managed install).
#[test]
fn rtf_to_pdf_via_libreoffice() {
    if missing(Tool::LibreOffice) {
        return;
    }
    let dir = scratch("rtf");
    let rtf = dir.join("memo.rtf");
    std::fs::write(&rtf, r"{\rtf1\ansi Hello from a managed LibreOffice.\par}").unwrap();

    let pdf = convert_file(&rtf, "pdf", 30).unwrap();
    assert_eq!(pdf, dir.join("memo.pdf"));
    assert!(std::fs::read(pdf).unwrap().starts_with(b"%PDF"));
}
