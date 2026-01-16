//! Persistence round-trip and edge case tests.
//!
//! Tests file I/O operations, JSON persistence, and settings round-trip.

use std::path::PathBuf;
use tempfile::TempDir;

use crate::persistence::{ensure_dir, load_json, save_json};
use crate::settings_store::{DataSourceMode, LogLevel, ProviderSettings, RefreshCadence, Settings};
use exactobar_core::ProviderKind;

// ============================================================================
// JSON Persistence Tests
// ============================================================================

#[tokio::test]
async fn test_save_and_load_json_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.json");

    let settings = Settings::default();

    // Save
    save_json(&file_path, &settings).await.unwrap();

    // Load
    let loaded: Settings = load_json(&file_path).await.unwrap();

    // Verify
    assert_eq!(loaded.refresh_cadence, settings.refresh_cadence);
    assert_eq!(loaded.debug_mode, settings.debug_mode);
    assert_eq!(loaded.enabled_providers, settings.enabled_providers);
}

#[tokio::test]
async fn test_save_creates_parent_directories() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir
        .path()
        .join("deeply")
        .join("nested")
        .join("path")
        .join("test.json");

    let data = serde_json::json!({"key": "value"});

    // Should create all parent directories
    let result = save_json(&nested_path, &data).await;
    assert!(result.is_ok());
    assert!(nested_path.exists());
}

#[tokio::test]
async fn test_load_nonexistent_file() {
    let file_path = PathBuf::from("/nonexistent/path/settings.json");

    let result: Result<Settings, _> = load_json(&file_path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ensure_dir_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new_directory");

    assert!(!new_dir.exists());

    ensure_dir(&new_dir).await.unwrap();

    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[tokio::test]
async fn test_ensure_dir_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("test_dir");

    // Create twice - should not fail
    ensure_dir(&dir_path).await.unwrap();
    ensure_dir(&dir_path).await.unwrap();

    assert!(dir_path.exists());
}

// ============================================================================
// Settings Persistence Tests
// ============================================================================

#[tokio::test]
async fn test_settings_full_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("settings.json");

    // Create non-default settings
    let mut settings = Settings {
        refresh_cadence: RefreshCadence::FiveMinutes,
        auto_refresh_on_wake: false,
        merge_icons: true,
        show_reset_countdown: false,
        debug_mode: true,
        log_level: LogLevel::Debug,
        selected_provider: Some(ProviderKind::Claude),
        ..Default::default()
    };

    // Add provider-specific settings
    let cursor_settings = ProviderSettings {
        source_mode: Some(DataSourceMode::Web),
        browser_preference: Some("chrome".to_string()),
        ..Default::default()
    };
    settings
        .provider_settings
        .insert(ProviderKind::Cursor, cursor_settings);

    // Enable some non-default providers
    settings.enabled_providers.insert(ProviderKind::Cursor);
    settings.enabled_providers.insert(ProviderKind::Gemini);
    settings.enabled_providers.remove(&ProviderKind::Codex);

    // Save and load
    save_json(&file_path, &settings).await.unwrap();
    let loaded: Settings = load_json(&file_path).await.unwrap();

    // Verify all fields preserved
    assert_eq!(loaded.refresh_cadence, RefreshCadence::FiveMinutes);
    assert!(!loaded.auto_refresh_on_wake);
    assert!(loaded.merge_icons);
    assert!(!loaded.show_reset_countdown);
    assert!(loaded.debug_mode);
    assert_eq!(loaded.log_level, LogLevel::Debug);
    assert_eq!(loaded.selected_provider, Some(ProviderKind::Claude));

    // Verify provider settings
    assert!(loaded.provider_settings.contains_key(&ProviderKind::Cursor));
    let cursor_loaded = loaded.provider_settings.get(&ProviderKind::Cursor).unwrap();
    assert_eq!(cursor_loaded.source_mode, Some(DataSourceMode::Web));
    assert_eq!(cursor_loaded.browser_preference, Some("chrome".to_string()));

    // Verify enabled providers
    assert!(loaded.enabled_providers.contains(&ProviderKind::Claude));
    assert!(loaded.enabled_providers.contains(&ProviderKind::Cursor));
    assert!(loaded.enabled_providers.contains(&ProviderKind::Gemini));
    assert!(!loaded.enabled_providers.contains(&ProviderKind::Codex));
}

#[tokio::test]
async fn test_settings_all_refresh_cadences() {
    let temp_dir = TempDir::new().unwrap();

    for cadence in RefreshCadence::all() {
        let file_path = temp_dir.path().join(format!("settings_{cadence:?}.json"));

        let settings = Settings {
            refresh_cadence: *cadence,
            ..Default::default()
        };

        save_json(&file_path, &settings).await.unwrap();
        let loaded: Settings = load_json(&file_path).await.unwrap();

        assert_eq!(loaded.refresh_cadence, *cadence, "Failed for {cadence:?}");
    }
}

#[tokio::test]
async fn test_settings_all_log_levels() {
    let temp_dir = TempDir::new().unwrap();

    let levels = vec![
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ];

    for level in levels {
        let file_path = temp_dir.path().join(format!("settings_{level:?}.json"));

        let settings = Settings {
            log_level: level,
            ..Default::default()
        };

        save_json(&file_path, &settings).await.unwrap();
        let loaded: Settings = load_json(&file_path).await.unwrap();

        assert_eq!(loaded.log_level, level);
    }
}

#[tokio::test]
async fn test_settings_all_providers_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("all_providers.json");

    let mut settings = Settings::default();

    // Enable all providers
    for kind in ProviderKind::all() {
        settings.enabled_providers.insert(*kind);
    }

    save_json(&file_path, &settings).await.unwrap();
    let loaded: Settings = load_json(&file_path).await.unwrap();

    assert_eq!(loaded.enabled_providers.len(), ProviderKind::all().len());
    for kind in ProviderKind::all() {
        assert!(loaded.enabled_providers.contains(kind));
    }
}

#[tokio::test]
async fn test_settings_no_providers_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("no_providers.json");

    let mut settings = Settings::default();
    settings.enabled_providers.clear();

    save_json(&file_path, &settings).await.unwrap();
    let loaded: Settings = load_json(&file_path).await.unwrap();

    assert!(loaded.enabled_providers.is_empty());
}

// ============================================================================
// Backward Compatibility Tests
// ============================================================================

#[tokio::test]
async fn test_load_minimal_json_uses_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("minimal.json");

    // Write minimal JSON
    tokio::fs::write(&file_path, "{}").await.unwrap();

    // Load - should use defaults for missing fields
    let loaded: Settings = load_json(&file_path).await.unwrap();

    // Should have default values
    assert_eq!(loaded.refresh_cadence, RefreshCadence::default());
    assert!(loaded.auto_refresh_on_wake); // Default is true
    assert!(loaded.merge_icons); // Default is true
}

#[tokio::test]
async fn test_load_json_with_unknown_fields() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("extra_fields.json");

    // Write JSON with unknown fields
    let json = r#"{
        "refresh_cadence": "two_minutes",
        "unknown_field_1": "value1",
        "unknown_field_2": 12345,
        "nested_unknown": {"key": "value"}
    }"#;
    tokio::fs::write(&file_path, json).await.unwrap();

    // Should not fail - unknown fields should be ignored
    let result: Result<Settings, _> = load_json(&file_path).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_atomic_write() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("atomic.json");

    let settings = Settings::default();

    // Save should be atomic (write to temp file, then rename)
    save_json(&file_path, &settings).await.unwrap();

    // The temp file should not exist after save
    let temp_path = file_path.with_extension("json.tmp");
    assert!(!temp_path.exists());

    // The final file should exist
    assert!(file_path.exists());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_save_large_settings() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.json");

    let mut settings = Settings::default();

    // Add settings for all providers
    for kind in ProviderKind::all() {
        let provider_settings = ProviderSettings {
            source_mode: Some(DataSourceMode::Auto),
            browser_preference: Some("firefox".to_string()),
            api_key_env: Some(format!("{kind:?}_API_KEY").to_uppercase()),
            ..Default::default()
        };
        settings.provider_settings.insert(*kind, provider_settings);
    }

    // Save and reload
    save_json(&file_path, &settings).await.unwrap();
    let loaded: Settings = load_json(&file_path).await.unwrap();

    assert_eq!(loaded.provider_settings.len(), ProviderKind::all().len());
}

#[tokio::test]
async fn test_unicode_in_settings() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("unicode.json");

    let mut settings = Settings::default();

    // Add unicode content - use string fields like cookie_header and browser_preference
    let provider_settings = ProviderSettings {
        cookie_header: Some("🚀 emoji test 日本語 中文".to_string()),
        browser_preference: Some("テスト ブラウザ".to_string()),
        ..Default::default()
    };
    settings
        .provider_settings
        .insert(ProviderKind::Claude, provider_settings);

    save_json(&file_path, &settings).await.unwrap();
    let loaded: Settings = load_json(&file_path).await.unwrap();

    let claude_settings = loaded.provider_settings.get(&ProviderKind::Claude).unwrap();
    assert_eq!(
        claude_settings.cookie_header,
        Some("🚀 emoji test 日本語 中文".to_string())
    );
    assert_eq!(
        claude_settings.browser_preference,
        Some("テスト ブラウザ".to_string())
    );
}
