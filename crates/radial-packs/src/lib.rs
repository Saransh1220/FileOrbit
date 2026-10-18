//! Modular converter installs: engines, the packs that need them, and the
//! installer that downloads, verifies and records them.

pub mod catalog;
pub mod extract;
pub mod fetch;
pub mod installer;
pub mod manifest;
pub mod paths;
pub mod platform;
pub mod state;
pub mod system;
