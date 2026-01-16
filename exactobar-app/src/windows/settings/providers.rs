//! Providers settings pane - helper types and functions.

use std::process::Command;

use exactobar_core::ProviderKind;
use exactobar_providers::ProviderRegistry;
use exactobar_store::{CookieSource, DataSourceMode};
use gpui::{Context, Hsla};

use crate::state::AppState;

// ============================================================================
// Provider Status
// ============================================================================

/// Status of a provider's availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    /// Provider is available and working
    Available,
    /// CLI tool not installed
    CliMissing,
    /// Authentication required
    AuthRequired,
    /// Check failed or unknown
    Unknown,
    /// Currently checking
    Checking,
}

impl ProviderStatus {
    /// Status indicator text (emoji/symbol).
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Available => "✓",
            Self::CliMissing => "✗",
            Self::AuthRequired => "⚠",
            Self::Unknown => "?",
            Self::Checking => "⟳",
        }
    }

    /// Status color (returns an Hsla).
    pub fn color(&self) -> Hsla {
        match self {
            Self::Available => Hsla {
                h: 120.0 / 360.0,
                s: 0.7,
                l: 0.4,
                a: 1.0,
            }, // Green
            Self::CliMissing => Hsla {
                h: 0.0,
                s: 0.7,
                l: 0.5,
                a: 1.0,
            }, // Red
            Self::AuthRequired => Hsla {
                h: 45.0 / 360.0,
                s: 0.9,
                l: 0.5,
                a: 1.0,
            }, // Yellow
            Self::Unknown => Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.5,
                a: 1.0,
            }, // Gray
            Self::Checking => Hsla {
                h: 210.0 / 360.0,
                s: 0.7,
                l: 0.5,
                a: 1.0,
            }, // Blue
        }
    }

    /// Tooltip text explaining the status.
    pub fn tooltip(&self) -> &'static str {
        match self {
            Self::Available => "Ready to use",
            Self::CliMissing => "CLI tool not installed",
            Self::AuthRequired => "Authentication required",
            Self::Unknown => "Status unknown",
            Self::Checking => "Checking availability...",
        }
    }
}

/// Detect the status of a provider by checking CLI availability.
///
/// This performs a quick check to see if the CLI tool exists on the system.
/// It does not make network calls or check authentication status.
pub fn detect_provider_status(provider: ProviderKind) -> ProviderStatus {
    // Get the CLI name for this provider
    let cli_name = match provider {
        ProviderKind::Codex => "codex",
        ProviderKind::Claude => "claude",
        ProviderKind::Copilot => "gh",
        ProviderKind::Gemini => "gcloud",
        ProviderKind::Kiro => {
            // Kiro can be either "kiro-cli" or "kiro"
            if which::which("kiro-cli").is_ok() || which::which("kiro").is_ok() {
                return ProviderStatus::Available;
            }
            return ProviderStatus::CliMissing;
        }
        // Web-only providers don't have CLIs
        ProviderKind::Cursor
        | ProviderKind::MiniMax
        | ProviderKind::Factory
        | ProviderKind::Augment => {
            return ProviderStatus::Unknown;
        }
        // API-key based providers
        ProviderKind::Synthetic => {
            // Check Keychain first, then env var
            if exactobar_store::has_api_key("synthetic")
                || std::env::var("SYNTHETIC_API_KEY").is_ok()
            {
                return ProviderStatus::Available;
            }
            return ProviderStatus::AuthRequired;
        }
        ProviderKind::Zai => {
            // Check Keychain for z.ai API key
            if exactobar_store::has_api_key("zai") {
                return ProviderStatus::Available;
            }
            return ProviderStatus::AuthRequired;
        }
        ProviderKind::VertexAI | ProviderKind::Antigravity => {
            // These use local credentials/probes
            return ProviderStatus::Unknown;
        }
    };

    // Check if CLI exists using the which crate
    if which::which(cli_name).is_err() {
        return ProviderStatus::CliMissing;
    }

    // CLI exists - assume available
    // (Full auth checking would require async operations)
    ProviderStatus::Available
}

/// Get the install command hint for a provider.
pub fn get_install_command(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Codex => "npm install -g @openai/codex",
        ProviderKind::Claude => "npm install -g @anthropic-ai/claude-code",
        ProviderKind::Copilot => "brew install gh && gh auth login",
        ProviderKind::Gemini => "brew install google-cloud-sdk",
        ProviderKind::Kiro => "npm install -g kiro-cli",
        ProviderKind::Synthetic => "Configure API key in Settings",
        ProviderKind::Zai => "Configure API key in Settings",
        _ => "See provider documentation",
    }
}

// ============================================================================
// API Key Support
// ============================================================================

/// Check if a provider requires an API key for authentication.
///
/// Returns `true` for providers that use API keys stored in the keychain,
/// as opposed to CLI-based auth (like Claude Code) or OAuth flows.
pub fn provider_needs_api_key(provider: ProviderKind) -> bool {
    matches!(
        provider,
        ProviderKind::Synthetic | ProviderKind::Zai | ProviderKind::Codex
    )
}

/// Get the keychain storage name for a provider's API key.
///
/// This returns the identifier used to store/retrieve the API key
/// from the system keychain via `exactobar_store::keychain`.
pub fn provider_api_key_name(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Synthetic => "synthetic",
        ProviderKind::Zai => "zai",
        ProviderKind::Codex => "codex",
        _ => "",
    }
}

/// Check if an API key is configured for a provider.
///
/// Checks both keychain storage and environment variables (for backward compatibility).
pub fn provider_has_api_key(provider: ProviderKind) -> bool {
    let keychain_name = provider_api_key_name(provider);
    if keychain_name.is_empty() {
        return false;
    }

    // Check keychain first
    if exactobar_store::has_api_key(keychain_name) {
        return true;
    }

    // Fall back to environment variables for backward compatibility
    match provider {
        ProviderKind::Synthetic => std::env::var("SYNTHETIC_API_KEY").is_ok(),
        ProviderKind::Zai => std::env::var("ZAI_API_KEY").is_ok(),
        ProviderKind::Codex => std::env::var("OPENAI_API_KEY").is_ok(),
        _ => false,
    }
}

/// Prompt for API key using osascript (native macOS dialog).
///
/// Shows a secure input dialog with hidden text entry.
/// Returns `Some(key)` if user entered a non-empty key and clicked Save,
/// `None` if cancelled or empty.
pub fn prompt_for_api_key(provider_name: &str) -> Option<String> {
    let script = format!(
        r#"
        set dialogResult to display dialog "Enter API key for {}:" default answer "" with hidden answer buttons {{"Cancel", "Save"}} default button "Save"
        if button returned of dialogResult is "Save" then
            return text returned of dialogResult
        else
            return ""
        end if
        "#,
        provider_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .ok()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !result.is_empty() {
            return Some(result);
        }
    }
    None
}

/// Async version of `prompt_for_api_key` that runs on a background thread.
///
/// Uses `smol::unblock` to run the blocking osascript command without
/// blocking the GPUI event loop.
pub async fn prompt_for_api_key_async(provider_name: &str) -> Option<String> {
    let name = provider_name.to_string();
    smol::unblock(move || prompt_for_api_key(&name)).await
}

// ============================================================================
// Provider Row Data
// ============================================================================

/// Provider row data for rendering.
pub struct ProviderRowData {
    pub provider: ProviderKind,
    pub is_enabled: bool,
    pub name: String,
    pub cli_name: String,
    pub is_primary: bool,
    pub supports_cookies: bool,
    pub supports_data_source: bool,
    pub current_cookie_source: CookieSource,
    pub current_data_source: Option<DataSourceMode>,
    /// Provider availability status
    pub status: ProviderStatus,
    /// Whether this provider needs an API key
    pub needs_api_key: bool,
    /// Whether an API key is currently configured
    pub has_api_key: bool,
    /// Keychain storage name for the API key
    pub api_key_name: &'static str,
}

/// Check if a provider supports cookie-based web fetching.
pub fn provider_supports_cookies(provider: ProviderKind) -> bool {
    matches!(
        provider,
        ProviderKind::Codex
            | ProviderKind::Claude
            | ProviderKind::Cursor
            | ProviderKind::Factory
            | ProviderKind::MiniMax
            | ProviderKind::Augment
    )
}

/// Check if a provider supports data source mode selection.
pub fn provider_supports_data_source(provider: ProviderKind) -> bool {
    matches!(provider, ProviderKind::Codex | ProviderKind::Claude)
}

/// Collect all provider data for rendering.
pub fn collect_provider_data<V: 'static>(cx: &Context<V>) -> Vec<ProviderRowData> {
    let state = cx.global::<AppState>();
    let settings = state.settings.read(cx);
    let all_providers = ProviderRegistry::all();

    all_providers
        .iter()
        .map(|desc| {
            let provider = desc.id;
            let is_enabled = settings.is_provider_enabled(provider);
            let supports_cookies = provider_supports_cookies(provider);
            let supports_data_source = provider_supports_data_source(provider);
            let current_cookie_source = settings.cookie_source(provider);
            let current_data_source = if supports_data_source {
                Some(match provider {
                    ProviderKind::Codex => settings.codex_data_source(),
                    ProviderKind::Claude => settings.claude_data_source(),
                    _ => DataSourceMode::Auto,
                })
            } else {
                None
            };

            // Detect provider status
            let status = detect_provider_status(provider);

            // API key info
            let needs_api_key = provider_needs_api_key(provider);
            let api_key_name = provider_api_key_name(provider);
            let has_api_key = provider_has_api_key(provider);

            ProviderRowData {
                provider,
                is_enabled,
                name: desc.display_name().to_string(),
                cli_name: desc.cli_name().to_string(),
                is_primary: desc.metadata.is_primary_provider,
                supports_cookies,
                supports_data_source,
                current_cookie_source,
                current_data_source,
                status,
                needs_api_key,
                has_api_key,
                api_key_name,
            }
        })
        .collect()
}

/// Cookie source options for the selector.
pub const COOKIE_SOURCES: [CookieSource; 6] = [
    CookieSource::Auto,
    CookieSource::Safari,
    CookieSource::Chrome,
    CookieSource::Arc,
    CookieSource::Firefox,
    CookieSource::Off,
];

/// Data source mode options for the selector.
pub const DATA_SOURCE_MODES: [DataSourceMode; 4] = [
    DataSourceMode::Auto,
    DataSourceMode::Cli,
    DataSourceMode::Web,
    DataSourceMode::Api,
];
