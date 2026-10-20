//! Several files dropped at once are one job: they convert one after another and
//! end in one summary, instead of a toast per file.

use std::path::{Path, PathBuf};

/// What happened to every file of a job, collected as they finish.
#[derive(Default, Debug)]
pub struct Outcome {
    pub outputs: Vec<PathBuf>,
    pub errors: Vec<String>,
}

/// How the whole job ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Summary {
    /// At least one file converted. `first` is what "Show" reveals; `failed` files did not convert.
    Done { first: PathBuf, converted: u32, failed: u32 },
    /// Nothing converted. The message is the first problem found.
    Failed { error: String },
}

pub fn summarize(outcome: &Outcome) -> Summary {
    match outcome.outputs.first() {
        Some(first) => Summary::Done {
            first: first.clone(),
            converted: outcome.outputs.len() as u32,
            failed: outcome.errors.len() as u32,
        },
        None => Summary::Failed {
            error: outcome.errors.first().cloned().unwrap_or_else(|| "nothing was converted".to_string()),
        },
    }
}

/// What the toast calls the job: the file's name, or "5 files".
pub fn job_name(inputs: &[PathBuf]) -> String {
    match inputs {
        [one] => file_name(one),
        many => format!("{} files", many.len()),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// "3 of 8: report.docx", shown while a batch runs.
pub fn progress_line(position: usize, total: usize, input: &Path) -> String {
    format!("{position} of {total}: {}", file_name(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn one_file_is_named_by_its_file_name() {
        assert_eq!(job_name(&paths(&["/a/report.docx"])), "report.docx");
    }

    #[test]
    fn several_files_are_counted() {
        assert_eq!(job_name(&paths(&["/a/1.docx", "/a/2.docx", "/a/3.docx"])), "3 files");
    }

    #[test]
    fn a_batch_where_everything_worked() {
        let outcome = Outcome { outputs: paths(&["/a/1.pdf", "/a/2.pdf"]), errors: vec![] };
        assert_eq!(summarize(&outcome), Summary::Done { first: "/a/1.pdf".into(), converted: 2, failed: 0 });
    }

    #[test]
    fn a_batch_with_some_failures_still_reports_what_worked() {
        let outcome = Outcome { outputs: paths(&["/a/1.pdf"]), errors: vec!["bad".into(), "worse".into()] };
        assert_eq!(summarize(&outcome), Summary::Done { first: "/a/1.pdf".into(), converted: 1, failed: 2 });
    }

    #[test]
    fn a_batch_where_nothing_worked_shows_the_first_problem() {
        let outcome = Outcome { outputs: vec![], errors: vec!["first problem".into(), "second".into()] };
        assert_eq!(summarize(&outcome), Summary::Failed { error: "first problem".into() });
    }

    #[test]
    fn progress_names_the_current_file() {
        assert_eq!(progress_line(3, 8, Path::new("/a/report.docx")), "3 of 8: report.docx");
    }
}
