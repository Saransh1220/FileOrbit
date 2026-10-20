//! PDF work through PDFium (the PDF library from Chrome), loaded from the
//! "PDF tools" pack. Everything here is about documents; the conversion
//! pipeline's entry points live in `engines/pdfium.rs`.

pub mod merge;
pub mod pictures;
pub mod plan;
pub mod render;
pub mod text;
pub mod thumbs;

use super::binaries::{self, Tool};
use pdfium_render::prelude::*;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// PDFium, bound to the library the "PDF tools" pack installed.
pub struct Library {
    pdfium: Pdfium,
}

/// PDFium can be bound only once per process, so the first conversion loads it and every
/// later one reuses it. (Binding it a second time fails with "already initialized".)
static SHARED: OnceLock<Library> = OnceLock::new();
/// Keeps two conversions that start together from both trying to bind it.
static LOADING: Mutex<()> = Mutex::new(());

impl Library {
    /// The shared PDFium, loading it from the managed pack on first use. Says which
    /// pack to install when it is missing.
    pub fn load() -> Result<&'static Library, String> {
        if let Some(library) = SHARED.get() {
            return Ok(library);
        }
        let _turn = LOADING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(library) = SHARED.get() {
            return Ok(library);
        }
        let path = binaries::find(Tool::Pdfium)?;
        let library = Self::bind(&path)?;
        Ok(SHARED.get_or_init(|| library))
    }

    fn bind(path: &Path) -> Result<Self, String> {
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
