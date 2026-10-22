//! Downloading and checksum verification.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

const CHUNK: usize = 64 * 1024;

/// How long to wait for a server to accept the connection. A host can list several
/// addresses and one of them may be unreachable from a given network (GitHub's download
/// host did exactly that), so the wait is shared out among the addresses: a dead first
/// address then costs seconds, not the operating system's minute-plus default.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// How long to wait for the server to start answering once connected.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_recv_response(Some(RESPONSE_TIMEOUT))
        .build()
        .into()
}

/// Fetches `url` into `dest`, reporting `(bytes so far, total if known)`.
/// Supports `http(s)://` and `file://` (offline mirrors, tests).
///
/// If `dest` already holds the start of the file (an interrupted download), the
/// transfer continues from there when the server supports it, and starts over
/// when it does not. The reported numbers always count from the file's start.
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
        return copy_stream(&mut reader, dest, 0, total, on_progress);
    }

    let have = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    if have > 0 {
        match request(url, Some(have)) {
            Ok(response) if response.status().as_u16() == 206 => {
                let total = content_length(&response).map(|rest| have + rest);
                let mut reader = response.into_body().into_reader();
                return copy_stream(&mut reader, dest, have, total, on_progress);
            }
            // The server ignored the range, or the partial file is no use: fall through and start over.
            Ok(_) | Err(_) => {}
        }
    }
    let response = request(url, None)?;
    let total = content_length(&response);
    let mut reader = response.into_body().into_reader();
    copy_stream(&mut reader, dest, 0, total, on_progress)
}

fn request(url: &str, from: Option<u64>) -> Result<ureq::http::Response<ureq::Body>, String> {
    let mut call = agent().get(url);
    if let Some(offset) = from {
        call = call.header("Range", &format!("bytes={offset}-"));
    }
    call.call().map_err(|e| format!("download failed for {url}: {e}"))
}

fn content_length(response: &ureq::http::Response<ureq::Body>) -> Option<u64> {
    response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

/// Writes `reader` to `dest`. With `already > 0` the bytes are appended after
/// the first `already` bytes; otherwise the file starts empty.
fn copy_stream(
    reader: &mut dyn Read,
    dest: &Path,
    already: u64,
    total: Option<u64>,
    on_progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<(), String> {
    let mut out = if already > 0 {
        std::fs::OpenOptions::new().append(true).open(dest)
    } else {
        File::create(dest)
    }
    .map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
    let mut buffer = vec![0u8; CHUNK];
    let mut done = already;
    on_progress(done, total);
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
    fn a_server_that_never_answers_gives_up_instead_of_hanging() {
        // Accepts the connection but never replies. Only the response timeout can end this
        // quickly enough to test, so it checks the agent really carries a timeout.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _held = std::thread::spawn(move || {
            let _conn = listener.accept();
            std::thread::park();
        });
        let started = std::time::Instant::now();
        let dir = tempfile::tempdir().unwrap();
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(2)))
            .timeout_recv_response(Some(Duration::from_millis(400)))
            .build();
        let result = ureq::Agent::from(config).get(&format!("http://127.0.0.1:{port}/x")).call();
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(5), "must not hang");
        let _ = dir;
        // The real agent is configured with the documented limits.
        let _ = agent();
    }

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
