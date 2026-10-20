//! Downloads are kept and reused: after a failed unpack, after a removal, and
//! across an interrupted transfer. Runs on every OS (no fake engines needed).

use radial_packs::installer::{Event, Installer};
use radial_packs::manifest::Manifest;
use radial_packs::{fetch, paths};
use std::cell::Cell;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const PLATFORM: &str = "test-platform";

fn make_zip(path: &Path, entries: &[&str]) {
    let mut writer = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    for entry in entries {
        writer.start_file(*entry, options).unwrap();
        writer.write_all(b"fake engine").unwrap();
    }
    writer.finish().unwrap();
}

fn manifest(url: &str, zip: &Path) -> Manifest {
    let sha = fetch::sha256_file(zip).unwrap();
    let size = std::fs::metadata(zip).unwrap().len();
    Manifest::parse(&format!(
        r#"{{"version":1,"engines":{{"ffmpeg":{{"version":"1.0","platforms":{{
          "{PLATFORM}":{{"url":"{url}","sha256":"{sha}","archive":"zip","bin":"bin/ffmpeg","size_bytes":{size}}}}}}}}}}}"#
    ))
    .unwrap()
}

fn packs(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

struct Counts {
    downloaded: Cell<u32>,
    cached: Cell<u32>,
}

impl Counts {
    fn new() -> Self {
        Self { downloaded: Cell::new(0), cached: Cell::new(0) }
    }
    fn record(&self, event: Event) {
        match event {
            Event::Downloading { .. } => self.downloaded.set(self.downloaded.get() + 1),
            Event::UsingCached { .. } => self.cached.set(self.cached.get() + 1),
            _ => {}
        }
    }
}

fn file_installer(zip: &Path, home: PathBuf) -> Installer {
    let url = url::Url::from_file_path(zip).unwrap().to_string();
    Installer::new(home, manifest(&url, zip), PLATFORM.into())
}

#[test]
fn a_failed_unpack_keeps_the_download_and_the_retry_reuses_it() {
    let dir = tempfile::tempdir().unwrap();
    let zip = dir.path().join("ffmpeg.zip");
    // The archive checks out but does not hold the executable, so unpacking "fails".
    make_zip(&zip, &["readme.txt"]);
    let installer = file_installer(&zip, dir.path().join("home"));

    let first = Counts::new();
    let err = installer.install(&packs(&["audio"]), &mut |e| first.record(e)).unwrap_err();
    assert!(err.contains("does not contain"), "{err}");
    assert!(err.contains("will not download it again"), "{err}");
    assert_eq!(first.cached.get(), 0);
    assert!(first.downloaded.get() > 0);

    let kept = paths::downloads_dir(installer.home()).join("ffmpeg-1.0.zip");
    assert!(kept.is_file(), "the verified download must survive a failed unpack");

    let second = Counts::new();
    let _ = installer.install(&packs(&["audio"]), &mut |e| second.record(e));
    assert_eq!(second.cached.get(), 1, "the second try reuses the file");
    assert_eq!(second.downloaded.get(), 0, "and does not download anything");
}

#[test]
fn reinstalling_after_a_removal_does_not_download_again() {
    let dir = tempfile::tempdir().unwrap();
    let zip = dir.path().join("ffmpeg.zip");
    make_zip(&zip, &["bin/ffmpeg"]);
    let installer = file_installer(&zip, dir.path().join("home"));

    installer.install(&packs(&["audio"]), &mut |_| {}).unwrap();
    installer.remove("audio").unwrap();

    let again = Counts::new();
    installer.install(&packs(&["audio"]), &mut |e| again.record(e)).unwrap();
    assert_eq!(again.cached.get(), 1);
    assert_eq!(again.downloaded.get(), 0);
    assert!(Installer::find_engine(installer.home(), "ffmpeg").is_some());
}

#[test]
fn a_corrupt_cached_file_is_replaced_not_trusted() {
    let dir = tempfile::tempdir().unwrap();
    let zip = dir.path().join("ffmpeg.zip");
    make_zip(&zip, &["bin/ffmpeg"]);
    let installer = file_installer(&zip, dir.path().join("home"));

    let kept = paths::downloads_dir(installer.home()).join("ffmpeg-1.0.zip");
    std::fs::create_dir_all(kept.parent().unwrap()).unwrap();
    std::fs::write(&kept, "not the real file").unwrap();

    let counts = Counts::new();
    installer.install(&packs(&["audio"]), &mut |e| counts.record(e)).unwrap();
    assert_eq!(counts.cached.get(), 0, "a file that fails its checksum is not reused");
    assert!(counts.downloaded.get() > 0);
}

#[test]
fn the_cache_can_be_measured_and_cleared() {
    let dir = tempfile::tempdir().unwrap();
    let zip = dir.path().join("ffmpeg.zip");
    make_zip(&zip, &["bin/ffmpeg"]);
    let installer = file_installer(&zip, dir.path().join("home"));
    assert_eq!(installer.cache_bytes(), 0);

    installer.install(&packs(&["audio"]), &mut |_| {}).unwrap();
    assert_eq!(installer.cache_bytes(), std::fs::metadata(&zip).unwrap().len());
    assert!(installer.engine_disk_bytes("ffmpeg") > 0);

    installer.clear_cache().unwrap();
    assert_eq!(installer.cache_bytes(), 0);
    // Clearing the cache never touches what is installed.
    assert!(Installer::find_engine(installer.home(), "ffmpeg").is_some());
    installer.clear_cache().unwrap();
}

/// A tiny HTTP server that serves `body`, honours `Range: bytes=N-`, and can
/// cut the first response short to simulate a dropped connection.
fn serve(body: Vec<u8>, cut_first_at: Option<usize>, saw_range: Arc<AtomicBool>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let mut first = true;
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let start = request
                .lines()
                .find_map(|l| l.to_ascii_lowercase().strip_prefix("range: bytes=").map(str::to_string))
                .and_then(|r| r.trim_end_matches('-').trim().parse::<usize>().ok());
            if start.is_some() {
                saw_range.store(true, Ordering::SeqCst);
            }
            let from = start.unwrap_or(0);
            let rest = &body[from..];
            let head = if start.is_some() {
                format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nConnection: close\r\n\r\n",
                    rest.len(),
                    from,
                    body.len() - 1,
                    body.len()
                )
            } else {
                format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", rest.len())
            };
            let _ = stream.write_all(head.as_bytes());
            if first {
                first = false;
                if let Some(cut) = cut_first_at {
                    let _ = stream.write_all(&rest[..cut.min(rest.len())]);
                    continue; // dropping the stream ends the response early
                }
            }
            let _ = stream.write_all(rest);
        }
    });
    format!("http://127.0.0.1:{port}/file.bin")
}

#[test]
fn an_interrupted_download_continues_where_it_stopped() {
    let body: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    let saw_range = Arc::new(AtomicBool::new(false));
    let url = serve(body.clone(), Some(70_000), saw_range.clone());
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("file.bin.part");

    // First attempt: the connection drops part-way, leaving a partial file.
    assert!(fetch::download(&url, &dest, &mut |_, _| {}).is_err());
    let partial = std::fs::metadata(&dest).unwrap().len();
    assert!(partial > 0 && partial < body.len() as u64, "partial file has {partial} bytes");

    // Second attempt: continues, and the progress counts from the start of the file.
    let mut first_report = None;
    let mut last = (0, None);
    fetch::download(&url, &dest, &mut |done, total| {
        first_report.get_or_insert(done);
        last = (done, total);
    })
    .unwrap();
    assert!(saw_range.load(Ordering::SeqCst), "the retry must ask for a range");
    assert_eq!(first_report, Some(partial));
    assert_eq!(last, (body.len() as u64, Some(body.len() as u64)));
    assert_eq!(std::fs::read(&dest).unwrap(), body);
}
