//! Shift+click is Explorer's "select a range" gesture, so pressing Shift on one of
//! several selected files makes Explorer shrink the selection to that file before
//! the drag begins. To still act on everything the user selected, the selection is
//! remembered while Shift goes down (before any click), and this rule decides
//! which of the two to use once the drag starts.
#![cfg_attr(not(windows), allow(dead_code))]

use std::path::PathBuf;

/// The files to act on, given the selection remembered before the press and the one
/// read after it.
///
/// - Explorer only collapsed the selection (every file now selected was selected before):
///   the remembered selection is what the user meant.
/// - Anything else (a file that was not selected before, a different window, no memory):
///   the current selection is, because the user has chosen something new.
pub fn choose(remembered: &[PathBuf], current: Vec<PathBuf>) -> Vec<PathBuf> {
    let collapsed = !remembered.is_empty()
        && !current.is_empty()
        && current.len() < remembered.len()
        && current.iter().all(|file| remembered.contains(file));
    if collapsed { remembered.to_vec() } else { current }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(|n| PathBuf::from(format!("C:\\docs\\{n}"))).collect()
    }

    #[test]
    fn a_collapsed_selection_gets_the_remembered_files_back() {
        let remembered = files(&["a.pdf", "b.pdf", "c.pdf"]);
        assert_eq!(choose(&remembered, files(&["b.pdf"])), remembered);
    }

    #[test]
    fn a_partly_collapsed_selection_is_also_restored() {
        let remembered = files(&["a.pdf", "b.pdf", "c.pdf"]);
        assert_eq!(choose(&remembered, files(&["a.pdf", "b.pdf"])), remembered);
    }

    #[test]
    fn an_unchanged_selection_is_just_used() {
        let remembered = files(&["a.pdf", "b.pdf"]);
        assert_eq!(choose(&remembered, files(&["a.pdf", "b.pdf"])), remembered);
    }

    #[test]
    fn dragging_a_file_that_was_not_selected_uses_the_new_selection() {
        let remembered = files(&["a.pdf", "b.pdf"]);
        let current = files(&["z.pdf"]);
        assert_eq!(choose(&remembered, current.clone()), current);
    }

    #[test]
    fn a_range_that_grew_the_selection_is_used_as_it_is() {
        let remembered = files(&["a.pdf"]);
        let current = files(&["a.pdf", "b.pdf", "c.pdf"]);
        assert_eq!(choose(&remembered, current.clone()), current);
    }

    #[test]
    fn with_nothing_remembered_the_current_selection_wins() {
        let current = files(&["a.pdf"]);
        assert_eq!(choose(&[], current.clone()), current);
    }

    #[test]
    fn an_emptied_selection_stays_empty_so_no_ring_opens() {
        assert!(choose(&files(&["a.pdf", "b.pdf"]), Vec::new()).is_empty());
    }
}
