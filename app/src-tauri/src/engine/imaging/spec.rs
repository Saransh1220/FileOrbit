//! What the user asked for. Plain data, shared with the window as JSON.

use serde::{Deserialize, Serialize};

/// How a picture is brought to a new shape when proportions differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Fit {
    /// Squeeze or stretch to the exact size (distorts).
    Stretch,
    /// Fill the frame and cut off what sticks out.
    Crop,
    /// Keep the whole picture and fill the rest with a background.
    Pad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Resize {
    /// Scale both sides by this percentage of the original.
    Percent { percent: u32 },
    /// Shrink to fit inside a box, keeping proportions. Never enlarges.
    Fit { width: u32, height: u32 },
    /// Shrink so the longer side is this many pixels. Never enlarges.
    LongEdge { pixels: u32 },
    /// Exactly this many pixels.
    Exact { width: u32, height: u32, fit: Fit },
    /// Change only the proportions (width:height), keeping the original pixels where possible.
    Aspect { width: u32, height: u32, fit: Fit },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Compress {
    /// A fixed quality, 1 to 100 (JPG only; PNG has none).
    Quality { quality: u8 },
    /// As good as possible while staying under this many bytes.
    TargetSize { max_bytes: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutFormat {
    Keep,
    Jpg,
    Png,
}

/// Any mix of the three parts; a part left out is simply not done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub resize: Option<Resize>,
    pub compress: Option<Compress>,
    pub format: OutFormat,
}

impl Job {
    /// The words added to the file name: "photo (resized, compressed).jpg".
    pub fn suffix(&self) -> &'static str {
        match (self.resize.is_some(), self.compress.is_some()) {
            (true, true) => " (resized, compressed)",
            (true, false) => " (resized)",
            (false, true) => " (compressed)",
            (false, false) => " (converted)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The window sends exactly this shape (see `app/src/imagetool/types.ts`).
    #[test]
    fn the_job_the_window_sends_parses() {
        let json = r#"{
            "resize": {"kind": "exact", "width": 800, "height": 600, "fit": "pad"},
            "compress": {"kind": "targetSize", "max_bytes": 300000},
            "format": "jpg"
        }"#;
        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.resize, Some(Resize::Exact { width: 800, height: 600, fit: Fit::Pad }));
        assert_eq!(job.compress, Some(Compress::TargetSize { max_bytes: 300_000 }));
        assert_eq!(job.format, OutFormat::Jpg);
    }

    #[test]
    fn every_resize_and_compress_kind_parses() {
        for json in [
            r#"{"kind":"percent","percent":50}"#,
            r#"{"kind":"fit","width":1,"height":2}"#,
            r#"{"kind":"longEdge","pixels":9}"#,
            r#"{"kind":"aspect","width":16,"height":9,"fit":"crop"}"#,
        ] {
            serde_json::from_str::<Resize>(json).unwrap_or_else(|e| panic!("{json}: {e}"));
        }
        serde_json::from_str::<Compress>(r#"{"kind":"quality","quality":80}"#).unwrap();
        let none: Job = serde_json::from_str(r#"{"resize":null,"compress":null,"format":"keep"}"#).unwrap();
        assert_eq!((none.resize, none.compress), (None, None));
    }
}
