//! Conversion engine: what can be converted (registry), how (engines) and
//! the job lifecycle around it. UI-agnostic apart from `jobs`.

pub mod binaries;
pub mod capabilities;
pub mod download_text;
pub mod engines;
pub mod jobs;
pub mod naming;
pub mod pdf;
pub mod pending;
pub mod process;
pub mod registry;
pub mod target;
