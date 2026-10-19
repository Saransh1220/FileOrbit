//! Human wording for a running download: size, speed and time left.

use std::time::Duration;

/// Speed and time left are only shown once enough has downloaded to be steady.
const MIN_ELAPSED: Duration = Duration::from_secs(3);

/// Whole megabytes, rounded to nearest.
pub fn mb(bytes: u64) -> u64 {
    (bytes + 500_000) / 1_000_000
}

/// "34 of 77 MB", or "34 MB" when the total is unknown.
fn amount(done: u64, total: Option<u64>) -> String {
    match total.filter(|t| *t > 0) {
        Some(t) => format!("{} of {} MB", mb(done), mb(t)),
        None => format!("{} MB", mb(done)),
    }
}

/// "42 s", "3 min" or "1 h 5 min", rounded up so it never promises too much.
pub fn time_left(seconds: u64) -> String {
    match seconds {
        0..=59 => format!("{} s", seconds.max(1)),
        60..=3599 => format!("{} min", seconds.div_ceil(60)),
        _ => format!("{} h {} min", seconds / 3600, (seconds % 3600).div_ceil(60)),
    }
}

/// The line under the toast title while an engine downloads, e.g.
/// "Downloading ffmpeg: 34 of 77 MB · 0.6 MB/s · 1 min left".
/// Speed is the average since the download began, which stays steady.
pub fn downloading(engine: &str, done: u64, total: Option<u64>, elapsed: Duration) -> String {
    let mut line = format!("Downloading {engine}: {}", amount(done, total));
    if elapsed < MIN_ELAPSED || done == 0 {
        return line;
    }
    let bytes_per_second = done as f64 / elapsed.as_secs_f64();
    line.push_str(&format!(" \u{b7} {:.1} MB/s", bytes_per_second / 1_000_000.0));
    if let Some(t) = total.filter(|t| *t > done) {
        let seconds = ((t - done) as f64 / bytes_per_second).ceil() as u64;
        line.push_str(&format!(" \u{b7} {} left", time_left(seconds)));
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn early_on_there_is_no_speed_yet() {
        let s = downloading("ffmpeg", 2_000_000, Some(77_000_000), Duration::from_secs(1));
        assert_eq!(s, "Downloading ffmpeg: 2 of 77 MB");
    }

    #[test]
    fn shows_speed_and_time_left() {
        // 20 MB in 10 s = 2 MB/s; 57 MB left = 28.5 s, rounded up to 29 s.
        let s = downloading("ffmpeg", 20_000_000, Some(77_000_000), Duration::from_secs(10));
        assert_eq!(s, "Downloading ffmpeg: 20 of 77 MB \u{b7} 2.0 MB/s \u{b7} 29 s left");
    }

    #[test]
    fn unknown_total_shows_speed_but_no_time_left() {
        let s = downloading("x", 10_000_000, None, Duration::from_secs(5));
        assert_eq!(s, "Downloading x: 10 MB \u{b7} 2.0 MB/s");
    }

    #[test]
    fn time_left_reads_naturally() {
        assert_eq!(time_left(0), "1 s");
        assert_eq!(time_left(59), "59 s");
        assert_eq!(time_left(61), "2 min");
        assert_eq!(time_left(3700), "1 h 2 min");
    }
}
