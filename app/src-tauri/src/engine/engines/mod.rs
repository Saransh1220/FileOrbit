mod ffmpeg;
mod libreoffice;
mod pandoc;
mod sips;

use super::binaries::Tool;
use super::target::Engine;
use std::path::Path;

/// Runs one conversion with the engine the registry chose.
pub fn run(
    engine: Engine,
    input: &Path,
    from_ext: &str,
    output: &Path,
    to_ext: &str,
    job_id: u64,
) -> Result<(), String> {
    match engine {
        Engine::Pandoc => pandoc::convert(input, from_ext, output, to_ext),
        Engine::LibreOffice => libreoffice::convert(input, output, to_ext, job_id),
        Engine::Ffmpeg => ffmpeg::convert(input, output, to_ext),
        Engine::Sips => sips::convert(input, output, to_ext),
    }
}

/// The program an engine runs.
pub fn tool_for(engine: Engine) -> Tool {
    match engine {
        Engine::Pandoc => Tool::Pandoc,
        Engine::LibreOffice => Tool::LibreOffice,
        Engine::Ffmpeg => Tool::Ffmpeg,
        Engine::Sips => Tool::Sips,
    }
}
