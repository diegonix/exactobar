//! Update available dialog window.
//!
//! Displays a modal dialog when a new version of ExactoBar is available,
//! showing version info, release notes, and download options.

use gpui::prelude::*;
use gpui::*;
use tracing::info;

use crate::updater::{UpdateCheckResult, open_release_page};

// ============================================================================
// Update Dialog
// ============================================================================

/// The update dialog window content.
pub struct UpdateDialog {
    /// Current installed version.
    pub current: String,
    /// Latest available version.
    pub latest: String,
    /// URL to the GitHub release page.
    pub release_url: String,
    /// Release notes/changelog (optional).
    pub release_notes: Option<String>,
}

impl UpdateDialog {
    /// Creates a new update dialog from an `UpdateCheckResult`.
    ///
    /// Returns `None` if the result is not an `UpdateAvailable` variant.
    pub fn new(result: &UpdateCheckResult) -> Option<Self> {
        if let UpdateCheckResult::UpdateAvailable {
            current,
            latest,
            release_url,
            release_notes,
            ..
        } = result
        {
            Some(Self {
                current: current.clone(),
                latest: latest.clone(),
                release_url: release_url.clone(),
                release_notes: release_notes.clone(),
            })
        } else {
            None
        }
    }
}

impl Render for UpdateDialog {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let release_url = self.release_url.clone();
        let current = self.current.clone();
        let latest = self.latest.clone();
        let has_notes = self.release_notes.is_some();
        let release_notes = self.release_notes.clone().unwrap_or_default();

        div()
            .size_full()
            .bg(hsla(0.0, 0.0, 0.1, 1.0))
            .text_color(white())
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            // Header with celebration emoji
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("🎉 Update Available!"),
            )
            // Version info section
            .child(Self::render_version_info(&current, &latest))
            // Release notes section (if available)
            .when(has_notes, |el| {
                el.child(Self::render_release_notes(release_notes.clone()))
            })
            // Action buttons
            .child(Self::render_buttons(release_url))
    }
}

impl UpdateDialog {
    /// Renders the version info section.
    fn render_version_info(current: &str, latest: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_base()
                    .child("A new version of ExactoBar is available!"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(hsla(0.0, 0.0, 0.7, 1.0))
                    .child(format!("Current: v{}  →  New: v{}", current, latest)),
            )
    }

    /// Renders the release notes section with scrollable content.
    fn render_release_notes(notes: String) -> impl IntoElement {
        div()
            .id("release-notes-scroll")
            .mt(px(8.0))
            .p(px(12.0))
            .rounded(px(8.0))
            .bg(hsla(0.0, 0.0, 0.15, 1.0))
            .max_h(px(150.0))
            .overflow_y_scroll()
            .child(
                div()
                    .text_sm()
                    .text_color(hsla(0.0, 0.0, 0.8, 1.0))
                    .child(notes),
            )
    }

    /// Renders the action buttons (Download and Later).
    fn render_buttons(release_url: String) -> impl IntoElement {
        div()
            .mt(px(8.0))
            .flex()
            .gap(px(12.0))
            // Download button - primary action
            .child(
                div()
                    .id("download-btn")
                    .px(px(16.0))
                    .py(px(8.0))
                    .rounded(px(6.0))
                    .bg(hsla(217.0 / 360.0, 0.9, 0.5, 1.0))
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(hsla(217.0 / 360.0, 0.9, 0.6, 1.0)))
                    .on_mouse_down(MouseButton::Left, move |_, window, _| {
                        info!(url = release_url.as_str(), "User clicked download update");
                        open_release_page(&release_url);
                        // Close the dialog after opening the page
                        window.remove_window();
                    })
                    .child("Download Update"),
            )
            // Later button - dismiss
            .child(
                div()
                    .id("later-btn")
                    .px(px(16.0))
                    .py(px(8.0))
                    .rounded(px(6.0))
                    .bg(hsla(0.0, 0.0, 0.2, 1.0))
                    .text_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(hsla(0.0, 0.0, 0.25, 1.0)))
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        info!("User dismissed update dialog");
                        window.remove_window();
                    })
                    .child("Later"),
            )
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Opens the update dialog window.
///
/// # Arguments
///
/// * `result` - The update check result (should be `UpdateAvailable`).
/// * `cx` - The application context.
pub fn show_update_dialog(result: &UpdateCheckResult, cx: &mut App) {
    let Some(dialog) = UpdateDialog::new(result) else {
        return;
    };

    info!(
        current = dialog.current.as_str(),
        latest = dialog.latest.as_str(),
        "Showing update dialog"
    );

    // Activate the app first (required for menu bar apps)
    cx.activate(true);

    let bounds = Bounds::centered(None, size(px(400.0), px(300.0)), cx);

    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from("ExactoBar Update")),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        focus: true,
        show: true,
        kind: WindowKind::Normal,
        is_movable: true,
        display_id: None,
        window_background: WindowBackgroundAppearance::Opaque,
        app_id: None,
        window_min_size: None,
        window_decorations: None,
        is_minimizable: true,
        is_resizable: false,
        tabbing_identifier: None,
    };

    match cx.open_window(options, |window, cx| {
        window.activate_window();
        cx.new(|_| dialog)
    }) {
        Ok(handle) => {
            info!("Update dialog opened successfully");
            // Ensure window is in front
            let any_handle: AnyWindowHandle = handle.into();
            let _ = cx.update_window(any_handle, |_, window, _| {
                window.activate_window();
            });
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to open update dialog");
        }
    }
}
