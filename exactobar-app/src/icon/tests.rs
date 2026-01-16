//! Tests for the icon rendering module.

use super::*;
use exactobar_core::UsageWindow;

// ============================================================================
// Basic Rendering Tests
// ============================================================================

#[test]
fn test_render_empty() {
    let renderer = IconRenderer::new();
    let icon = renderer.render(ProviderKind::Codex, None, false, None, None);

    assert_eq!(icon.width, ICON_WIDTH);
    assert_eq!(icon.height, ICON_HEIGHT);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_with_snapshot() {
    let renderer = IconRenderer::new();

    let mut snapshot = UsageSnapshot::new();
    snapshot.primary = Some(UsageWindow::new(25.0));
    snapshot.secondary = Some(UsageWindow::new(50.0));

    let icon = renderer.render(ProviderKind::Claude, Some(&snapshot), false, None, None);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_stale() {
    let renderer = IconRenderer::new();
    let mut snapshot = UsageSnapshot::new();
    snapshot.primary = Some(UsageWindow::new(25.0));

    let icon = renderer.render(ProviderKind::Claude, Some(&snapshot), true, None, None);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_with_status() {
    let renderer = IconRenderer::new();
    let icon = renderer.render(
        ProviderKind::Codex,
        None,
        false,
        Some(StatusIndicator::Minor),
        None,
    );
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_credits() {
    let renderer = IconRenderer::new();
    let icon = renderer.render_credits(ProviderKind::Codex, Some(75.0), false);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_loading() {
    let renderer = IconRenderer::new();
    let icon = renderer.render_loading(ProviderKind::Codex, 0.5);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_error() {
    let renderer = IconRenderer::new();
    let icon = renderer.render_error(ProviderKind::Codex);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_to_png() {
    let renderer = IconRenderer::new();
    let icon = renderer.render(ProviderKind::Codex, None, false, None, None);
    let png = icon.to_png();

    // PNG magic bytes
    assert!(png.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
}

#[test]
fn test_colored_mode() {
    let renderer = IconRenderer::new().with_mode(RenderMode::Colored);
    let icon = renderer.render(ProviderKind::Claude, None, false, None, None);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_template_mode() {
    let renderer = IconRenderer::new().with_mode(RenderMode::Template);
    let icon = renderer.render(ProviderKind::Claude, None, false, None, None);
    assert!(!icon.data.is_empty());
}

// ============================================================================
// Codex Eye Tests
// ============================================================================

#[test]
fn test_render_codex_eye_default() {
    let renderer = IconRenderer::new();
    let icon = renderer.render(ProviderKind::Codex, None, false, None, None);

    assert_eq!(icon.width, ICON_WIDTH);
    assert_eq!(icon.height, ICON_HEIGHT);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_codex_eye_with_usage() {
    let renderer = IconRenderer::new();

    let mut snapshot = UsageSnapshot::new();
    snapshot.primary = Some(UsageWindow::new(30.0)); // 70% remaining

    let icon = renderer.render(ProviderKind::Codex, Some(&snapshot), false, None, None);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_codex_eye_blinking() {
    let renderer = IconRenderer::new();

    // Test various blink phases
    for phase in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let animation = IconAnimationState::with_blink(phase);
        let icon = renderer.render(ProviderKind::Codex, None, false, None, Some(&animation));
        assert!(!icon.data.is_empty(), "Failed at blink phase {}", phase);
    }
}

#[test]
fn test_render_codex_eye_fully_closed() {
    let renderer = IconRenderer::new();
    let animation = IconAnimationState::with_blink(1.0);

    assert!(animation.is_closed());
    assert!(!animation.is_open());

    let icon = renderer.render(ProviderKind::Codex, None, false, None, Some(&animation));
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_codex_eye_fully_open() {
    let renderer = IconRenderer::new();
    let animation = IconAnimationState::with_blink(0.0);

    assert!(!animation.is_closed());
    assert!(animation.is_open());

    let icon = renderer.render(ProviderKind::Codex, None, false, None, Some(&animation));
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_codex_eye_colored_mode() {
    let renderer = IconRenderer::new().with_mode(RenderMode::Colored);
    let icon = renderer.render(ProviderKind::Codex, None, false, None, None);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_codex_eye_stale() {
    let renderer = IconRenderer::new();
    let mut snapshot = UsageSnapshot::new();
    snapshot.primary = Some(UsageWindow::new(50.0));

    let icon = renderer.render(ProviderKind::Codex, Some(&snapshot), true, None, None);
    assert!(!icon.data.is_empty());
}

#[test]
fn test_render_codex_eye_with_status_indicator() {
    let renderer = IconRenderer::new();
    let icon = renderer.render(
        ProviderKind::Codex,
        None,
        false,
        Some(StatusIndicator::Major),
        None,
    );
    assert!(!icon.data.is_empty());
}
