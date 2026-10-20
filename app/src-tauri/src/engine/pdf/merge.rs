//! Putting several PDFs and pictures into one PDF, in the order given.

use super::{pictures, Library};
use crate::engine::selection;
use std::path::{Path, PathBuf};

/// How many pages the merged PDF got.
#[derive(Debug, PartialEq, Eq)]
pub struct Merged {
    pub pages: u32,
}

/// Writes `output` holding every input's pages, one input after another. A PDF
/// contributes all its pages; a picture becomes one page. Fails, naming the file,
/// on the first input that cannot be read, and then writes nothing.
pub fn merge(library: &Library, inputs: &[PathBuf], output: &Path) -> Result<Merged, String> {
    if inputs.len() < 2 {
        return Err("choose at least two files to merge".into());
    }
    let mut document = library.create()?;
    for input in inputs {
        let ext = crate::engine::registry::normalized_ext(input).unwrap_or_default();
        if ext == "pdf" {
            let source = library.open(input, None)?;
            if source.pages().len() == 0 {
                return Err(format!("{} has no pages", name_of(input)));
            }
            document
                .pages_mut()
                .append(&source)
                .map_err(|e| format!("cannot add the pages of {}: {e}", name_of(input)))?;
        } else if selection::is_picture(&ext) {
            pictures::append_picture(&mut document, input)?;
        } else {
            return Err(format!("{} is not a PDF or a picture", name_of(input)));
        }
    }
    let pages = document.pages().len() as u32;
    document.save_to_file(output).map_err(|e| format!("cannot write the merged PDF: {e}"))?;
    Ok(Merged { pages })
}

fn name_of(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}
