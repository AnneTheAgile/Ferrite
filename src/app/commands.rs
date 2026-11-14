//! Command registry for the command palette.
//!
//! Provides a unified list of all executable commands with metadata
//! (display name, category, shortcut hint, icon) for the command palette UI.
//!
//! The list includes both built-in [`ShortcutCommand`]s and user-defined
//! editor macros. Macros carry a `macro_name` and are labelled by that name.

use crate::config::{Macro, ShortcutCommand};

/// What selecting a palette row should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    /// Run a built-in shortcut command.
    Shortcut(ShortcutCommand),
    /// Run a user-defined macro by name.
    Macro(String),
}

/// A command entry for the palette, combining an action with its display
/// metadata and an optional icon.
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    /// The action to run when this row is selected.
    pub action: PaletteAction,
    /// The icon glyph shown at the left of the row.
    pub icon: &'static str,
}

impl PaletteCommand {
    /// The label shown in the row. For a macro this is its configured name.
    #[must_use]
    pub fn label(&self) -> &str {
        match &self.action {
            PaletteAction::Shortcut(cmd) => cmd.display_name(),
            PaletteAction::Macro(name) => name,
        }
    }

    /// The category grouping used when the query is empty.
    #[must_use]
    pub fn category(&self) -> &'static str {
        match &self.action {
            PaletteAction::Shortcut(cmd) => cmd.category(),
            PaletteAction::Macro(_) => "Macro",
        }
    }

    /// The shortcut command this row maps to, when it is a built-in command.
    ///
    /// Macros have no key binding, so this returns `None` and the palette
    /// renders no shortcut hint for them.
    #[must_use]
    pub fn shortcut_command(&self) -> Option<ShortcutCommand> {
        match &self.action {
            PaletteAction::Shortcut(cmd) => Some(*cmd),
            PaletteAction::Macro(_) => None,
        }
    }
}

/// Build the full list of commands available in the palette.
///
/// Excludes CommandPalette itself (no recursion). Appends one row per
/// user-defined macro, in configuration order.
#[must_use]
pub fn all_palette_commands(macros: &[Macro]) -> Vec<PaletteCommand> {
    let mut commands: Vec<PaletteCommand> = ShortcutCommand::all()
        .iter()
        .filter(|cmd| !matches!(cmd, ShortcutCommand::CommandPalette))
        .map(|&command| PaletteCommand {
            action: PaletteAction::Shortcut(command),
            icon: icon_for_command(command),
        })
        .collect();

    for m in macros {
        commands.push(PaletteCommand {
            action: PaletteAction::Macro(m.name.clone()),
            icon: crate::ui::phosphor_icons::NOTE_PENCIL,
        });
    }

    commands
}

fn icon_for_command(cmd: ShortcutCommand) -> &'static str {
    use crate::ui::phosphor_icons::{
        ARROWS_IN, ARROW_CLOCKWISE, ARROW_COUNTER_CLOCKWISE, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT,
        ARROW_UP, BRACKETS_CURLY, CARET_DOWN, CARET_RIGHT, CARET_UP, CLIPBOARD, CODE, CORNERS_OUT,
        EYE, FILE_MAGNIFYING_GLASS, FILE_PDF, FILE_PLUS, FILE_TEXT, FLOPPY_DISK, FOLDERS, GEAR,
        GLOBE, IMAGE, INFO, LIGHTNING, LINK, LIST, LIST_BULLETS, LIST_CHECKS, LIST_NUMBERS, LOCK,
        MAGNIFYING_GLASS, MAGNIFYING_GLASS_MINUS, MAGNIFYING_GLASS_PLUS, MINUS, NOTE_PENCIL,
        PALETTE, PENCIL, PIPE, PLUS, PRINTER, SCROLL, TERMINAL_WINDOW, TEXT_ALIGN_LEFT, TEXT_B,
        TEXT_H_ONE, TEXT_ITALIC, TEXT_T, TRASH, TREE, X,
    };

    match cmd {
        // File
        ShortcutCommand::Save | ShortcutCommand::SaveAs => FLOPPY_DISK,
        ShortcutCommand::Open => FILE_TEXT,
        ShortcutCommand::New => FILE_PLUS,
        ShortcutCommand::NewTab => PLUS,
        ShortcutCommand::NewWindow => NOTE_PENCIL,
        ShortcutCommand::CloseTab => X,
        ShortcutCommand::Reload => ARROW_CLOCKWISE,
        ShortcutCommand::OpenWorkspace | ShortcutCommand::CloseWorkspace => FOLDERS,
        // Navigation
        ShortcutCommand::NextTab => ARROW_RIGHT,
        ShortcutCommand::PrevTab => ARROW_LEFT,
        ShortcutCommand::GoToLine => LIST_NUMBERS,
        ShortcutCommand::QuickOpen => LIGHTNING,
        // View
        ShortcutCommand::ToggleViewMode => EYE,
        ShortcutCommand::CycleTheme => PALETTE,
        ShortcutCommand::ToggleZenMode => ARROWS_IN,
        ShortcutCommand::ToggleWordWrap => TEXT_ALIGN_LEFT,
        ShortcutCommand::TogglePreviewLock => LOCK,
        ShortcutCommand::ToggleFullscreen => CORNERS_OUT,
        ShortcutCommand::ToggleOutline => LIST,
        ShortcutCommand::ToggleFileTree => TREE,
        ShortcutCommand::TogglePipeline => PIPE,
        ShortcutCommand::ToggleTerminal => TERMINAL_WINDOW,
        ShortcutCommand::ToggleProductivityHub => LIST_CHECKS,
        ShortcutCommand::ZoomIn => MAGNIFYING_GLASS_PLUS,
        ShortcutCommand::ZoomOut => MAGNIFYING_GLASS_MINUS,
        ShortcutCommand::ResetZoom => MAGNIFYING_GLASS,
        // Edit
        ShortcutCommand::Undo => ARROW_COUNTER_CLOCKWISE,
        ShortcutCommand::Redo => ARROW_CLOCKWISE,
        ShortcutCommand::DeleteLine => TRASH,
        ShortcutCommand::DuplicateLine => CLIPBOARD,
        ShortcutCommand::MoveLineUp => ARROW_UP,
        ShortcutCommand::MoveLineDown => ARROW_DOWN,
        ShortcutCommand::SelectNextOccurrence => TEXT_T,
        // Search
        ShortcutCommand::Find => MAGNIFYING_GLASS,
        ShortcutCommand::FindReplace => MAGNIFYING_GLASS,
        ShortcutCommand::FindNext => CARET_DOWN,
        ShortcutCommand::FindPrev => CARET_UP,
        ShortcutCommand::SearchInFiles => FILE_MAGNIFYING_GLASS,
        // Format
        ShortcutCommand::FormatBold => TEXT_B,
        ShortcutCommand::FormatItalic => TEXT_ITALIC,
        ShortcutCommand::FormatInlineCode => CODE,
        ShortcutCommand::FormatCodeBlock => BRACKETS_CURLY,
        ShortcutCommand::FormatLink => LINK,
        ShortcutCommand::FormatImage => IMAGE,
        ShortcutCommand::FormatBlockquote => NOTE_PENCIL,
        ShortcutCommand::FormatBulletList => LIST_BULLETS,
        ShortcutCommand::FormatNumberedList => LIST_NUMBERS,
        ShortcutCommand::FormatHeading1 => TEXT_H_ONE,
        ShortcutCommand::FormatHeading2
        | ShortcutCommand::FormatHeading3
        | ShortcutCommand::FormatHeading4
        | ShortcutCommand::FormatHeading5
        | ShortcutCommand::FormatHeading6 => TEXT_H_ONE,
        // Folding
        ShortcutCommand::FoldAll => CARET_RIGHT,
        ShortcutCommand::UnfoldAll => CARET_DOWN,
        ShortcutCommand::ToggleFoldAtCursor => MINUS,
        // Other
        ShortcutCommand::CommandPalette => MAGNIFYING_GLASS,
        ShortcutCommand::OpenSettings => GEAR,
        ShortcutCommand::OpenAbout => INFO,
        ShortcutCommand::ExportHtml => GLOBE,
        ShortcutCommand::ExportPdf => FILE_PDF,
        ShortcutCommand::PrintPreview => PRINTER,
        ShortcutCommand::InsertToc => SCROLL,
        ShortcutCommand::ToggleFrontmatter => PENCIL,
    }
}
