//! Locating converter programs.
//!
//! The managed install comes first; the machine's own copy (searched in
//! well-known folders too, since a GUI app has a minimal PATH) is the fallback.

use radial_packs::installer::Installer;
use radial_packs::paths;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub enum Tool {
    Pandoc,
    Ffmpeg,
    LibreOffice,
    /// The PDF library (a shared library, loaded rather than run).
    Pdfium,
    Sips,
}

impl Tool {
    fn exe_name(self) -> &'static str {
        match self {
            Tool::Pandoc => "pandoc",
            Tool::Ffmpeg => "ffmpeg",
            Tool::LibreOffice => "soffice",
            Tool::Pdfium => "pdfium",
            Tool::Sips => "sips",
        }
    }

    /// Engine id in the `radial` catalog, if this tool is installable that way.
    fn engine_id(self) -> Option<&'static str> {
        match self {
            Tool::Pandoc => Some("pandoc"),
            Tool::Ffmpeg => Some("ffmpeg"),
            Tool::LibreOffice => Some("libreoffice"),
            Tool::Pdfium => Some("pdfium"),
            Tool::Sips => None,
        }
    }

    /// What to tell the user when the tool is missing.
    fn install_hint(self) -> &'static str {
        match self {
            Tool::Pandoc => "Run `radial install` and choose \"Text & Markup\".",
            Tool::Ffmpeg => "Run `radial install` and choose Audio, Video or Images.",
            Tool::LibreOffice => "Run `radial install` and choose Documents.",
            Tool::Pdfium => "Run `radial install` and choose PDF tools.",
            Tool::Sips => "sips is built into macOS.",
        }
    }
}

/// Managed install first (what `radial install` set up), then the machine's
/// own copy (developer fallback).
pub fn find(tool: Tool) -> Result<PathBuf, String> {
    if let Some(path) = tool
        .engine_id()
        .and_then(|id| Installer::find_engine(&paths::default_home(), id))
    {
        return Ok(path);
    }
    find_on_system(tool)
}

fn find_on_system(tool: Tool) -> Result<PathBuf, String> {
    radial_packs::system::find(tool.exe_name())
        .ok_or_else(|| format!("{} is not installed. {}", tool.exe_name(), tool.install_hint()))
}
