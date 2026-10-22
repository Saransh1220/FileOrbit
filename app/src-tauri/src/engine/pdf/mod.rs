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
use std::cell::Cell;
use std::ops::Deref;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// PDFium, bound to the library the "PDF tools" pack installed.
pub struct Library {
    pdfium: Pdfium,
}

/// PDFium can be bound only once per process, so the first conversion loads it and every
/// later one reuses it. (Binding it a second time fails with "already initialized".)
static SHARED: OnceLock<Library> = OnceLock::new();
/// Keeps two conversions that start together from both trying to bind it.
static LOADING: Mutex<()> = Mutex::new(());
/// PDFium is not built for several threads working in it at once (on macOS two
/// conversions at the same moment crash the process), so one operation at a time
/// holds this, whatever started it: a conversion, a merge or the editor.
static IN_USE: Mutex<()> = Mutex::new(());

thread_local! {
    /// How many `LibraryGuard`s this thread holds. Only the first takes the lock, so a
    /// thread that loads the library again while already using it carries on instead of
    /// waiting for itself forever (a frozen app is worse than any crash).
    static HELD_BY_THIS_THREAD: Cell<u32> = const { Cell::new(0) };
}

/// PDFium, held for one operation. Dropping it lets the next operation in.
/// It derefs to [`Library`], so it is used exactly like one. Guards held by one thread
/// are expected to be dropped innermost first, which scoped variables do by themselves.
pub struct LibraryGuard {
    library: &'static Library,
    /// Present only on the thread's outermost guard.
    _turn: Option<MutexGuard<'static, ()>>,
}

impl Drop for LibraryGuard {
    fn drop(&mut self) {
        HELD_BY_THIS_THREAD.with(|held| held.set(held.get() - 1));
    }
}

impl Deref for LibraryGuard {
    type Target = Library;
    fn deref(&self) -> &Library {
        self.library
    }
}

impl Library {
    /// The shared PDFium for one operation, loading it from the managed pack on first
    /// use. Waits if another thread is using it. Says which pack to install when it is
    /// missing. Calling it again on a thread that already holds it is fine.
    pub fn load() -> Result<LibraryGuard, String> {
        let library = Self::shared()?;
        let outermost = HELD_BY_THIS_THREAD.with(|held| held.get() == 0);
        let turn = outermost.then(|| IN_USE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()));
        HELD_BY_THIS_THREAD.with(|held| held.set(held.get() + 1));
        Ok(LibraryGuard { library, _turn: turn })
    }

    fn shared() -> Result<&'static Library, String> {
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
        let bindings = Pdfium::bind_to_library(path).map_err(|e| {
            let raw = e.to_string();
            // The system's own text can run to many lines; keep all of it in the log and show the short version.
            eprintln!("[pdf] cannot load {}: {raw}", path.display());
            describe_load_error(path, &raw)
        })?;
        Ok(Self { pdfium: Pdfium::new(bindings) })
    }

    pub fn open<'a>(&'a self, path: &Path, password: Option<&str>) -> Result<PdfDocument<'a>, String> {
        self.pdfium.load_pdf_from_file(path, password).map_err(|e| describe_open_error(path, e))
    }

    pub fn create(&self) -> Result<PdfDocument<'_>, String> {
        self.pdfium.create_new_pdf().map_err(|e| format!("cannot create a PDF: {e}"))
    }
}

/// How long a "cannot load" message may get before it is cut.
const MAX_LOAD_ERROR: usize = 220;

/// A short, readable version of the (often many-line) error from loading the PDF library.
fn describe_load_error(path: &Path, raw: &str) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    // macOS refuses a library its rules do not allow: say what that means instead of dumping the policy text.
    if raw.contains("library load disallowed by system policy") || raw.contains("not valid for use in process") {
        return format!(
            "macOS blocked {name} from loading because this build of the app is not allowed to load add-on libraries. \
             Reinstall the app from a build signed with that permission."
        );
    }
    let first_line = raw.lines().next().unwrap_or(raw).trim();
    let short: String = first_line.chars().take(MAX_LOAD_ERROR).collect();
    let cut = if first_line.chars().count() > MAX_LOAD_ERROR { "…" } else { "" };
    format!("cannot load the PDF library {name}: {short}{cut}")
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

#[cfg(test)]
mod tests {
    use super::describe_load_error;
    use std::path::Path;

    const MAC_POLICY_ERROR: &str = "dlopen(/x/libpdfium.dylib, 0x0002): tried: '/x/libpdfium.dylib' (code signature in <UUID> '/private/x/libpdfium.dylib' not valid for use in process: library load disallowed by system policy), '/System/Volumes/Preboot/Cryptexes/OS/x/libpdfium.dylib' (no such file), '/x/libpdfium.dylib' (code signature in <UUID> not valid for use in process)";

    #[test]
    fn the_macos_signing_refusal_gets_a_plain_explanation_not_the_policy_dump() {
        let text = describe_load_error(Path::new("/x/libpdfium.dylib"), MAC_POLICY_ERROR);
        assert!(text.contains("macOS blocked libpdfium.dylib"), "{text}");
        assert!(text.len() < 300 && !text.contains("tried:"), "{text}");
    }

    #[test]
    fn any_other_long_error_is_cut_to_one_short_line() {
        let long = format!("first line {}\nsecond line that must not appear", "x".repeat(900));
        let text = describe_load_error(Path::new("/x/pdfium.dll"), &long);
        assert!(text.chars().count() < 300, "{}", text.chars().count());
        assert!(text.ends_with('…') && !text.contains("second line"), "{text}");
    }

    #[test]
    fn a_short_error_passes_through_whole() {
        let text = describe_load_error(Path::new("/x/pdfium.dll"), "The specified module could not be found.");
        assert_eq!(text, "cannot load the PDF library pdfium.dll: The specified module could not be found.");
    }
}
