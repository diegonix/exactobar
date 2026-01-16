//! Application windows.

#![allow(dead_code)]

pub mod settings;
pub mod update;

pub use update::show_update_dialog;

use gpui::*;
use std::sync::Mutex;
use tracing::info;

use settings::SettingsWindow;

/// Global handle to the settings window (if open).
static SETTINGS_WINDOW: Mutex<Option<AnyWindowHandle>> = Mutex::new(None);

/// Opens the settings window, or focuses it if already open.
pub fn open_settings(cx: &mut App) {
    // Check if window already exists and is still valid
    {
        let guard = SETTINGS_WINDOW.lock().unwrap();
        if let Some(handle) = *guard {
            // Try to focus the existing window
            if cx
                .update_window(handle, |_, window, _| {
                    window.activate_window();
                })
                .is_ok()
            {
                info!("Focused existing settings window");
                cx.activate(true);
                return;
            }
            // Window was closed, continue to create new one
        }
    }

    info!("Opening settings window");

    // CRITICAL: For menu bar apps, we must activate the app first!
    cx.activate(true);

    let bounds = Bounds::centered(None, size(px(700.0), px(500.0)), cx);

    let options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("ExactoBar Settings".into()),
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
        window_min_size: Some(size(px(500.0), px(400.0))),
        window_decorations: None,
        is_minimizable: true,
        is_resizable: true,
        tabbing_identifier: None,
    };

    let result = cx.open_window(options, |window, cx| {
        window.activate_window();
        cx.new(|_| SettingsWindow::new())
    });

    match result {
        Ok(handle) => {
            info!("Settings window opened successfully");
            let any_handle: AnyWindowHandle = handle.into();

            // Store the handle
            {
                let mut guard = SETTINGS_WINDOW.lock().unwrap();
                *guard = Some(any_handle);
            }

            let _ = cx.update_window(any_handle, |_, window, _| {
                window.activate_window();
            });
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to open settings window");
        }
    }
}

/// Clear the settings window handle (call when window closes).
pub fn clear_settings_window() {
    let mut guard = SETTINGS_WINDOW.lock().unwrap();
    *guard = None;
}
