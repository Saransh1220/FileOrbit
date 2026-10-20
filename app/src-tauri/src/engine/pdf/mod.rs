//! PDF work through PDFium (the PDF library from Chrome), loaded from the
//! "PDF tools" pack. Everything here is about documents; the conversion
//! pipeline's entry points live in `engines/pdfium.rs`.

use super::binaries::{self, Tool};
use pdfium_render::prelude::*;
use std::path::Path;

/// PDFium, bound to the library the "PDF tools" pack installed.
pub struct Library {
    pdfium: Pdfium,
}

impl Library {
    /// Loads the managed PDFium library, or says which pack to install.
    pub fn load() -> Result<Self, String> {
        let path = binaries::find(Tool::Pdfium)?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self, String> {
        let bindings = Pdfium::bind_to_library(path)
            .map_err(|e| format!("cannot load the PDF library at {}: {e}", path.display()))?;
        Ok(Self { pdfium: Pdfium::new(bindings) })
    }

    pub fn open<'a>(&'a self, path: &Path, password: Option<&str>) -> Result<PdfDocument<'a>, String> {
        self.pdfium.load_pdf_from_file(path, password).map_err(|e| describe_open_error(path, e))
    }

    pub fn create(&self) -> Result<PdfDocument<'_>, String> {
        self.pdfium.create_new_pdf().map_err(|e| format!("cannot create a PDF: {e}"))
    }
}

/// A friendly sentence for the ways opening a PDF fails.
fn describe_open_error(path: &Path, error: PdfiumError) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    match error {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => {
            format!("{name} is protected by a password")
        }
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FormatError) => {
            format!("{name} is damaged or is not a PDF file")
        }
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FileError) => {
            format!("{name} could not be read")
        }
        other => format!("cannot open {name}: {other}"),
    }
}
