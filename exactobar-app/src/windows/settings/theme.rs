//! Theme support for the settings window.
//!
//! Provides light and dark mode colors that adapt to the system appearance.

use gpui::Hsla;

/// Theme colors for the settings window.
#[derive(Clone, Copy, Debug)]
pub struct SettingsTheme {
    /// Main background color
    pub bg: Hsla,
    /// Surface/card background color
    pub surface: Hsla,
    /// Border color for dividers and outlines
    pub border: Hsla,
    /// Primary text color
    pub text_primary: Hsla,
    /// Secondary/muted text color
    pub text_muted: Hsla,
    /// Selected item background
    pub selected: Hsla,
    /// Hover state background
    pub hover: Hsla,
    /// Link color
    pub link: Hsla,
    /// Brand accent color (used in About pane)
    pub brand: Hsla,
    /// Code/monospace background
    pub code_bg: Hsla,
    /// Warning color (yellow/amber)
    pub warning: Hsla,
    /// Error/danger color (red)
    pub error: Hsla,
    /// Success color (green)
    pub success: Hsla,
}

impl SettingsTheme {
    /// Light theme colors (default macOS light appearance)
    pub fn light() -> Self {
        Self {
            bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.97,
                a: 1.0,
            },
            surface: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.99,
                a: 1.0,
            },
            border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.9,
                a: 1.0,
            },
            text_primary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.1,
                a: 1.0,
            },
            text_muted: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.5,
                a: 1.0,
            },
            selected: Hsla {
                h: 217.0 / 360.0,
                s: 0.91,
                l: 0.95,
                a: 1.0,
            },
            hover: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.95,
                a: 1.0,
            },
            link: Hsla {
                h: 217.0 / 360.0,
                s: 0.91,
                l: 0.6,
                a: 1.0,
            },
            brand: Hsla {
                h: 160.0 / 360.0,
                s: 0.82,
                l: 0.35,
                a: 1.0,
            },
            code_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.95,
                a: 1.0,
            },
            warning: Hsla {
                h: 45.0 / 360.0,
                s: 0.9,
                l: 0.5,
                a: 1.0,
            },
            error: Hsla {
                h: 0.0,
                s: 0.7,
                l: 0.5,
                a: 1.0,
            },
            success: Hsla {
                h: 120.0 / 360.0,
                s: 0.7,
                l: 0.4,
                a: 1.0,
            },
        }
    }

    /// Dark theme colors (macOS dark appearance)
    pub fn dark() -> Self {
        Self {
            bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.12,
                a: 1.0,
            },
            surface: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.16,
                a: 1.0,
            },
            border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            },
            text_primary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.95,
                a: 1.0,
            },
            text_muted: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.6,
                a: 1.0,
            },
            selected: Hsla {
                h: 217.0 / 360.0,
                s: 0.7,
                l: 0.35,
                a: 1.0,
            },
            hover: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.22,
                a: 1.0,
            },
            link: Hsla {
                h: 217.0 / 360.0,
                s: 0.8,
                l: 0.65,
                a: 1.0,
            },
            brand: Hsla {
                h: 160.0 / 360.0,
                s: 0.72,
                l: 0.45,
                a: 1.0,
            },
            code_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.2,
                a: 1.0,
            },
            warning: Hsla {
                h: 45.0 / 360.0,
                s: 0.85,
                l: 0.55,
                a: 1.0,
            },
            error: Hsla {
                h: 0.0,
                s: 0.7,
                l: 0.55,
                a: 1.0,
            },
            success: Hsla {
                h: 120.0 / 360.0,
                s: 0.65,
                l: 0.45,
                a: 1.0,
            },
        }
    }
}
