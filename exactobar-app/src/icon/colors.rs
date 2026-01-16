//! Color management for icon rendering.
//!
//! This module contains the [`IconColors`] struct and helper functions
//! for managing colors in both template (grayscale) and colored modes.

use exactobar_core::ProviderKind;
use tiny_skia::{Color, Paint};

/// Color palette for icon rendering.
pub struct IconColors {
    pub track: Color,      // Bar background
    pub fill_stale: Color, // Fill when stale
    pub good: Color,       // >50% remaining
    pub warning: Color,    // 20-50% remaining
    pub danger: Color,     // <20% remaining
    pub loading: Color,    // Loading animation
}

impl IconColors {
    /// Template mode colors (grayscale for macOS template images).
    pub fn template(stale: bool) -> Self {
        let opacity = if stale { 140 } else { 204 }; // ~55% or ~80%

        Self {
            track: Color::from_rgba8(0, 0, 0, 102),       // 40% opacity
            fill_stale: Color::from_rgba8(0, 0, 0, 140),  // 55% opacity
            good: Color::from_rgba8(0, 0, 0, opacity),    // 80% or 55%
            warning: Color::from_rgba8(0, 0, 0, opacity), // Same for template
            danger: Color::from_rgba8(0, 0, 0, opacity),  // Same for template
            loading: Color::from_rgba8(0, 0, 0, 128),     // 50% opacity
        }
    }

    /// Colored mode with provider brand colors.
    pub fn colored(provider: ProviderKind, stale: bool) -> Self {
        let brand = provider_brand_color(provider);
        let alpha_mult = if stale { 0.7 } else { 1.0 };

        Self {
            track: Color::from_rgba8(80, 80, 80, 180),
            fill_stale: with_alpha(brand, 0.6),
            good: with_alpha(brand, alpha_mult),
            warning: with_alpha(Color::from_rgba8(255, 193, 7, 255), alpha_mult),
            danger: with_alpha(Color::from_rgba8(244, 67, 54, 255), alpha_mult),
            loading: Color::from_rgba8(150, 150, 150, 200),
        }
    }
}

/// Gets the brand color for a provider.
pub fn provider_brand_color(provider: ProviderKind) -> Color {
    match provider {
        ProviderKind::Codex => Color::from_rgba8(16, 163, 127, 255), // OpenAI green
        ProviderKind::Claude => Color::from_rgba8(204, 119, 68, 255), // Anthropic orange/tan
        ProviderKind::Cursor => Color::from_rgba8(138, 92, 246, 255), // Purple
        ProviderKind::Gemini => Color::from_rgba8(66, 133, 244, 255), // Google blue
        ProviderKind::Copilot => Color::from_rgba8(36, 41, 46, 255), // GitHub dark
        ProviderKind::Factory => Color::from_rgba8(255, 107, 107, 255), // Red
        ProviderKind::VertexAI => Color::from_rgba8(66, 133, 244, 255), // Google blue
        ProviderKind::Zai => Color::from_rgba8(100, 100, 100, 255),  // Gray
        ProviderKind::Augment => Color::from_rgba8(75, 0, 130, 255), // Indigo
        ProviderKind::Kiro => Color::from_rgba8(255, 165, 0, 255),   // Orange
        ProviderKind::MiniMax => Color::from_rgba8(0, 191, 255, 255), // Deep sky blue
        ProviderKind::Antigravity => Color::from_rgba8(148, 0, 211, 255), // Violet
        ProviderKind::Synthetic => Color::from_rgba8(0, 204, 179, 255), // Teal/cyan
    }
}

/// Adjusts color alpha.
pub fn with_alpha(color: Color, alpha: f64) -> Color {
    Color::from_rgba(
        color.red(),
        color.green(),
        color.blue(),
        (color.alpha() as f64 * alpha) as f32,
    )
    .unwrap_or(color)
}

/// Creates a paint with the given color.
pub fn create_paint(color: Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    paint
}
