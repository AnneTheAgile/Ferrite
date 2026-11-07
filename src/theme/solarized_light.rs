//! Solarized Light Theme
//!
//! Ethan Schoonover's [Solarized](https://ethanschoonover.com/solarized/) light
//! variant, wired into Ferrite as a third selectable theme.
//!
//! # Why this file is thin
//!
//! Ferrite's palette lives in [`super::ThemeColors`] (`BaseColors`,
//! `TextColors`, `EditorThemeColors`, `SyntaxColors`, `UiColors`), and
//! [`super::light::visuals_from_palette`] turns any palette into egui
//! `Visuals`. Solarized Light therefore needs no visualiser of its own: it is
//! a light theme (`ThemeColors::is_dark()` is false because base3 is #FDF6E3),
//! so it reuses the light widget rules and only swaps colours.
//!
//! # Palette summary
//!
//! | Role | Solarized | Hex |
//! |---|---|---|
//! | editor / panel background | base3 | #FDF6E3 |
//! | raised surfaces, code blocks | base2 | #EEE8D5 |
//! | body text | base01 | #586E75 |
//! | secondary text | base00 | #657B83 |
//! | borders | base01 / base1 | #586E75 / #93A1A1 |
//! | accent, headings, links | blue | #268BD2 |

// Allow dead code - exports mirror light.rs for future use
#![allow(dead_code)]

use eframe::egui::{Color32, Visuals};

use super::{light, ThemeColors};

/// Build Solarized Light `Visuals` from an already-built palette.
pub(crate) fn visuals_from_palette(colors: &ThemeColors) -> Visuals {
    light::visuals_from_palette(colors)
}

/// Ferrite Solarized Light egui visuals with the given Ferrite accent.
///
/// Note: the persisted accent colour (`Settings::accent_color`) overrides the
/// palette's own blue for selection, headings and widget accents — exactly as
/// it does for the Light theme. For the intended look, set the accent to
/// `#268BD2` (38, 139, 210).
pub fn create_solarized_light_visuals(accent: Color32) -> Visuals {
    let mut palette = ThemeColors::solarized_light();
    palette.apply_user_accent(accent);
    visuals_from_palette(&palette)
}

/// Solarized Light visuals with the palette's own blue accent, ignoring the
/// user accent setting.
pub fn create_solarized_light_visuals_defaults() -> Visuals {
    visuals_from_palette(&ThemeColors::solarized_light())
}

/// Get the Solarized Light theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors::solarized_light()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;

    #[test]
    fn test_solarized_is_light_mode() {
        let visuals = create_solarized_light_visuals(crate::theme::accent::default_accent());
        assert!(!visuals.dark_mode);
    }

    #[test]
    fn test_solarized_background_is_base3() {
        let visuals = create_solarized_light_visuals_defaults();
        assert_eq!(visuals.panel_fill, Color32::from_rgb(253, 246, 227));
        assert_eq!(visuals.window_fill, Color32::from_rgb(253, 246, 227));
    }

    #[test]
    fn test_solarized_palette_is_not_dark() {
        let colors = colors();
        assert!(!colors.is_dark());
    }

    #[test]
    fn test_solarized_differs_from_plain_light() {
        let solarized = colors();
        let light = ThemeColors::light();
        assert_ne!(solarized.base.background, light.base.background);
        assert_ne!(solarized.text.primary, light.text.primary);
    }

    #[test]
    fn test_solarized_text_contrast_on_base3() {
        // body text (base01) on base3 must stay readable: channel sum is a
        // crude but stable proxy for "clearly darker than the background".
        let c = colors();
        let bg = c.base.background;
        let fg = c.text.primary;
        let bg_sum = u32::from(bg.r()) + u32::from(bg.g()) + u32::from(bg.b());
        let fg_sum = u32::from(fg.r()) + u32::from(fg.g()) + u32::from(fg.b());
        assert!(bg_sum - fg_sum > 300, "text too close to background");
    }

    #[test]
    fn test_solarized_selection_visible() {
        let visuals = create_solarized_light_visuals_defaults();
        assert_ne!(visuals.selection.bg_fill, visuals.panel_fill);
    }

    #[test]
    fn test_solarized_resolves_through_from_theme() {
        let visuals = eframe::egui::Visuals::light();
        let resolved = ThemeColors::from_theme(
            Theme::SolarizedLight,
            &visuals,
            crate::theme::accent::default_accent(),
        );
        assert_eq!(resolved.base.background, Color32::from_rgb(253, 246, 227));
    }

    #[test]
    fn test_theme_serializes_with_hyphen() {
        assert_eq!(
            serde_json::to_string(&Theme::SolarizedLight).unwrap(),
            "\"solarized-light\""
        );
        assert_eq!(
            serde_json::from_str::<Theme>("\"solarized-light\"").unwrap(),
            Theme::SolarizedLight
        );
    }
}
