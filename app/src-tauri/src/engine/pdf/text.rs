//! Reading the text out of a PDF.

use pdfium_render::prelude::*;

/// Pages are separated by a blank line and a form feed, the traditional page break in plain text.
const PAGE_BREAK: &str = "\n\n\u{c}\n";

/// All the text of the document, page by page. Errors for a PDF with no text at all,
/// which is almost always a scan, so the user is told why instead of getting an empty file.
pub fn extract(document: &PdfDocument<'_>) -> Result<String, String> {
    let mut pages = Vec::new();
    for page in document.pages().iter() {
        let text = page.text().map_err(|e| format!("cannot read the page text: {e}"))?;
        pages.push(text.all());
    }
    join_pages(&pages)
}

fn join_pages(pages: &[String]) -> Result<String, String> {
    if pages.iter().all(|p| p.trim().is_empty()) {
        return Err("this PDF has no text to extract. It may be a scan (a picture of a page), which needs text recognition that is not available yet".into());
    }
    Ok(pages.iter().map(|p| p.trim_end()).collect::<Vec<_>>().join(PAGE_BREAK))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_are_separated_by_a_page_break() {
        let text = join_pages(&["one\r\n".into(), "two".into()]).unwrap();
        assert_eq!(text, "one\n\n\u{c}\ntwo");
    }

    #[test]
    fn a_pdf_without_text_explains_why() {
        let error = join_pages(&["  ".into(), "\n".into()]).unwrap_err();
        assert!(error.contains("scan"), "{error}");
    }

    #[test]
    fn a_document_with_one_blank_page_among_text_pages_is_fine() {
        assert!(join_pages(&["hello".into(), "".into()]).is_ok());
    }
}
