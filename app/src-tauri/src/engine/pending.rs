//! Conversions waiting for the user to install a missing pack.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct PendingJob {
    pub input: PathBuf,
    pub target: String,
    pub pack: &'static str,
}

/// Keyed by job id. Held in Tauri state so the "Install" button can find the
/// job it belongs to.
#[derive(Default)]
pub struct PendingJobs(Mutex<HashMap<u64, PendingJob>>);

impl PendingJobs {
    pub fn insert(&self, id: u64, job: PendingJob) {
        self.0.lock().unwrap().insert(id, job);
    }

    /// Removes and returns the job, so an Install click can only be used once.
    pub fn take(&self, id: u64) -> Option<PendingJob> {
        self.0.lock().unwrap().remove(&id)
    }

    /// Forgets every waiting job (the user dismissed the prompt).
    pub fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(pack: &'static str) -> PendingJob {
        PendingJob { input: "/a/b.docx".into(), target: "pdf".into(), pack }
    }

    #[test]
    fn a_job_can_be_taken_once() {
        let pending = PendingJobs::default();
        pending.insert(7, job("documents"));
        assert_eq!(pending.take(7).unwrap().pack, "documents");
        assert!(pending.take(7).is_none());
    }

    #[test]
    fn dismissing_forgets_everything() {
        let pending = PendingJobs::default();
        pending.insert(1, job("audio"));
        pending.insert(2, job("text"));
        pending.clear();
        assert!(pending.take(1).is_none() && pending.take(2).is_none());
    }
}
