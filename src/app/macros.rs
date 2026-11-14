//! Replayable editor macros.
//!
//! Phase one is a fixed command vocabulary: a macro is a named list of
//! [`MacroStep`]s stored in the config. There is no input-event recorder, and
//! no text capture. Replay is deterministic and testable.
//!
//! Line endings are handled explicitly: a `\n` inside an [`MacroStep::Insert`]
//! is translated to the active tab's own line ending, so a CRLF file stays
//! CRLF and an LF file stays LF. The buffer is never assumed to already hold a
//! particular ending.

use super::FerriteApp;
use crate::config::{Macro, MacroStep};
use crate::state::LineEnding;
use crate::string_utils::{
    char_index_to_byte_index, rope_char_index_at_line_start, rope_line_col_to_char_index,
};
use log::{debug, warn};

impl FerriteApp {
    /// Run a macro by name on the active tab.
    ///
    /// Does nothing when no macro has that name, or when there is no active
    /// tab. Both cases are logged rather than treated as errors.
    pub(crate) fn handle_run_macro(&mut self, name: &str) {
        let Some(macro_def) = self
            .state
            .settings
            .macros
            .iter()
            .find(|m| m.name == name)
            .cloned()
        else {
            warn!("Macro not found: {name}");
            return;
        };
        self.run_macro(&macro_def);
    }

    /// Replay a macro's steps against the active tab.
    ///
    /// The edit is applied to the tab's text and then recorded for undo, so a
    /// macro is one undo step. Cursor and selection are reset to the end of the
    /// last step.
    pub(crate) fn run_macro(&mut self, macro_def: &Macro) {
        let Some(tab) = self.state.active_tab_mut() else {
            return;
        };

        let old_content = tab.content.clone();
        let old_cursor = tab.cursors.primary().head;
        let ending = tab.line_ending;
        let eol = ending.as_str();

        let (line, col) = tab.cursor_position;
        let mut content = tab.content.clone();
        let mut cursor = rope_line_col_to_char_index(&content, line, col);
        let mut cursor_line = line;

        for step in &macro_def.steps {
            match step {
                MacroStep::MoveToLineStart => {
                    cursor = rope_char_index_at_line_start(&content, cursor_line);
                }
                MacroStep::MoveToLineEnd => {
                    let start = rope_char_index_at_line_start(&content, cursor_line);
                    let lines = LineEnding::split_lines(&content);
                    let line_len = lines.get(cursor_line).map_or(0, |l| l.chars().count());
                    cursor = start + line_len;
                }
                MacroStep::Insert(text) => {
                    // Translate the macro's portable `\n` to this tab's ending.
                    let text = text.replace('\n', eol);
                    let byte_idx = char_index_to_byte_index(&content, cursor);
                    content.insert_str(byte_idx, &text);
                    // One `\n` per new line, including in a CRLF pair.
                    let newlines = text.matches('\n').count();
                    cursor += text.chars().count();
                    cursor_line += newlines;
                }
            }
        }

        let cursor = cursor.min(content.chars().count());
        tab.content = content;
        tab.pending_cursor_restore = Some(cursor);
        tab.cursors
            .set_single(crate::state::Selection::cursor(cursor));
        tab.sync_cursor_from_primary();
        tab.record_edit(old_content, old_cursor);
        tab.mark_content_edited();
        debug!("Ran macro '{}'", macro_def.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MacroStep;

    fn tab_with(content: &str, line_ending: LineEnding, line: usize, col: usize) -> crate::state::Tab {
        let mut tab = crate::state::Tab::new(1);
        tab.content = content.to_string();
        tab.line_ending = line_ending;
        tab.cursor_position = (line, col);
        tab.cursors
            .set_single(crate::state::Selection::cursor(rope_line_col_to_char_index(
                content, line, col,
            )));
        tab
    }

    /// The step sequence and the resulting text for the handoff's "comment
    /// line" macro, checked against the string directly so it does not depend
    /// on the app or an egui context.
    fn apply_steps(content: &str, ending: LineEnding, line: usize, col: usize, steps: &[MacroStep]) -> String {
        let eol = ending.as_str();
        let mut content = content.to_string();
        let mut cursor = rope_line_col_to_char_index(&content, line, col);
        let mut cursor_line = line;
        for step in steps {
            match step {
                MacroStep::MoveToLineStart => {
                    cursor = rope_char_index_at_line_start(&content, cursor_line);
                }
                MacroStep::MoveToLineEnd => {
                    let start = rope_char_index_at_line_start(&content, cursor_line);
                    let lines = LineEnding::split_lines(&content);
                    let line_len = lines.get(cursor_line).map_or(0, |l| l.chars().count());
                    cursor = start + line_len;
                }
                MacroStep::Insert(text) => {
                    let text = text.replace('\n', eol);
                    let byte_idx = char_index_to_byte_index(&content, cursor);
                    content.insert_str(byte_idx, &text);
                    let newlines = text.matches('\n').count();
                    cursor += text.chars().count();
                    cursor_line += newlines;
                }
            }
        }
        content
    }

    #[test]
    fn comment_line_macro_lf() {
        // A single line with no trailing newline: the macro prefixes it and
        // appends a space plus this tab's newline.
        let macro_def = Macro::comment_line();
        let out = apply_steps("hello world", LineEnding::Lf, 0, 4, &macro_def.steps);
        assert_eq!(out, "-- hello world \n");
    }

    #[test]
    fn comment_line_macro_crlf() {
        // Same, but the appended newline must be CRLF.
        let macro_def = Macro::comment_line();
        let out = apply_steps("hello world", LineEnding::Crlf, 0, 4, &macro_def.steps);
        assert_eq!(out, "-- hello world \r\n");
    }

    #[test]
    fn move_to_line_start_then_insert() {
        let steps = vec![MacroStep::MoveToLineStart, MacroStep::Insert(">> ".into())];
        let out = apply_steps("alpha\nbeta", LineEnding::Lf, 1, 2, &steps);
        assert_eq!(out, "alpha\n>> beta");
    }

    #[test]
    fn insert_does_not_disturb_other_lines() {
        let steps = vec![MacroStep::MoveToLineStart, MacroStep::Insert("-- ".into())];
        let out = apply_steps("one\ntwo\nthree", LineEnding::Lf, 1, 3, &steps);
        assert_eq!(out, "one\n-- two\nthree");
    }

    #[test]
    fn tab_helper_is_consistent() {
        let tab = tab_with("a\r\nb", LineEnding::Crlf, 1, 1);
        assert_eq!(tab.cursor_position, (1, 1));
    }
}
