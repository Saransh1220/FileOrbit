//! Downloading and checksum verification.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const CHUNK: usize = 64 * 1024;

/// Fetches `url` into `dest`, reporting `(bytes so far, total if known)`.
/// Supports `http(s)://` and `file://` (offline mirrors, tests).
pub fn download(
    url: &str,
    dest: &Path,
    on_progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    if url.starts_with("file://") {
        let source = url::Url::parse(url)
            .ok()
            .and_then(|u| u.to_file_path().ok())
            .ok_or_else(|| format!("bad file URL: {url}"))?;
        let total = std::fs::metadata(&source).map(|m| m.len()).ok();
        let mut reader = File::open(&source).map_err(|e| format!("cannot open {}: {e}", source.display()))?;
        return copy_stream(&mut reader, dest, total, on_progress);
    }

    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("download failed for {url}: {e}"))?;
    let total = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    let mut reader = response.into_body().into_reader();
    copy_stream(&mut reader, dest, total, on_progress)
}

fn copy_stream(
    reader: &mut dyn Read,
    dest: &Path,
    total: Option<u64>,
    on_progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<(), String> {
    let mut out = File::create(dest).map_err(|e| format!("cannot create {}: {e}", dest.display()))?;
    let mut buffer = vec![0u8; CHUNK];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buffer).map_err(|e| format!("download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        out.write_all(&buffer[..n]).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
        done += n as u64;
        on_progress(done, total);
    }
    out.flush().map_err(|e| e.to_string())
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Errors unless the file's SHA-256 equals `expected` (case-insensitive).
pub fn verify(path: &Path, expected: &str) -> Result<(), String> {
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(format!("checksum mismatch: expected {expected}, got {actual}. The download is corrupt or has been tampered with"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_known_content() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "abc").unwrap();
        // SHA-256 of "abc"
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(sha256_file(&file).unwrap(), expected);
        assert!(verify(&file, &expected.to_uppercase()).is_ok());
        assert!(verify(&file, "00").unwrap_err().contains("mismatch"));
    }

    #[test]
    fn copies_a_file_url_and_reports_progress() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.bin");
        std::fs::write(&src, vec![7u8; 200_000]).unwrap();
        let url = url::Url::from_file_path(&src).unwrap().to_string();
        let dest = dir.path().join("out/dest.bin");
        let mut last = (0, None);
        download(&url, &dest, &mut |d, t| last = (d, t)).unwrap();
        assert_eq!(last, (200_000, Some(200_000)));
        assert_eq!(std::fs::read(&dest).unwrap().len(), 200_000);
    }

    #[test]
    fn missing_file_url_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = url::Url::from_file_path(dir.path().join("not-here.bin")).unwrap().to_string();
        let err = download(&missing, &dir.path().join("x"), &mut |_, _| {}).unwrap_err();
        assert!(err.contains("cannot open"), "{err}");
    }
}
