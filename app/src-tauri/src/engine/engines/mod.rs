mod ffmpeg;
mod libreoffice;
mod pandoc;
mod pdfium;
mod sips;

use super::binaries::Tool;
use super::target::Engine;
use std::path::{Path, PathBuf};

/// Runs one conversion with the engine the registry chose. Returns what was written:
/// `output`, unless the engine had to put the result somewhere else (a folder of pictures).
pub fn run(
    engine: Engine,
    input: &Path,
    from_ext: &str,
    output: &Path,
    to_ext: &str,
    job_id: u64,
) -> Result<PathBuf, String> {
    let written = |done: Result<(), String>| done.map(|()| output.to_path_buf());
    match engine {
        Engine::Pandoc => written(pandoc::convert(input, from_ext, output, to_ext)),
        Engine::LibreOffice => written(libreoffice::convert(input, from_ext, output, to_ext, job_id)),
        Engine::Ffmpeg => written(ffmpeg::convert(input, output, to_ext)),
        Engine::Sips => written(sips::convert(input, output, to_ext)),
        Engine::Pdfium => pdfium::convert(input, from_ext, output, to_ext),
    }
}

/// The program an engine runs.
pub fn tool_for(engine: Engine) -> Tool {
    match engine {
        Engine::Pandoc => Tool::Pandoc,
        Engine::LibreOffice => Tool::LibreOffice,
        Engine::Ffmpeg => Tool::Ffmpeg,
        Engine::Sips => Tool::Sips,
        Engine::Pdfium => Tool::Pdfium,
    }
}
