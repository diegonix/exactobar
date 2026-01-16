//! Auto-update system for ExactoBar.
//!
//! Checks GitHub Releases for new versions and offers to update.
//! This module uses the GitHub API to fetch release information
//! and compares semver versions to determine if an update is available.
//!
//! Note: We use `reqwest::blocking` with `smol::unblock()` because GPUI uses
//! the smol async runtime, not Tokio (which reqwest's async client requires).

use semver::Version;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, error, info, warn};

/// GitHub repository owner for releases
const GITHUB_OWNER: &str = "janfeddersen";

/// GitHub repository name for releases
const GITHUB_REPO: &str = "exactobar";

/// Current version from Cargo.toml (set at compile time)
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Global flag to prevent concurrent update checks.
/// Uses atomic operations for thread-safe access.
static CHECKING_UPDATE: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Update Check Result
// ============================================================================

/// Result of an update check operation.
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// A newer version is available on GitHub.
    UpdateAvailable {
        /// The currently installed version.
        current: String,
        /// The latest available version.
        latest: String,
        /// URL to the GitHub release page.
        release_url: String,
        /// Direct download URL for macOS asset (if available).
        download_url: Option<String>,
        /// Release notes/changelog (if available).
        release_notes: Option<String>,
    },
    /// Already running the latest version.
    UpToDate,
    /// Update check failed with an error.
    Error(String),
}

// ============================================================================
// Public API
// ============================================================================

/// Checks for updates asynchronously.
///
/// This function is safe to call from any context - it prevents
/// concurrent update checks using an atomic flag.
///
/// # Returns
///
/// An `UpdateCheckResult` indicating whether an update is available,
/// the app is up to date, or an error occurred.
pub async fn check_for_updates() -> UpdateCheckResult {
    // Prevent concurrent checks using atomic swap
    if CHECKING_UPDATE.swap(true, Ordering::SeqCst) {
        return UpdateCheckResult::Error("Update check already in progress".to_string());
    }

    let result = do_check_for_updates().await;

    // Reset the flag when done
    CHECKING_UPDATE.store(false, Ordering::SeqCst);

    result
}

/// Opens the release page in the default browser.
///
/// # Arguments
///
/// * `url` - The URL to open (typically the GitHub release page).
#[cfg(target_os = "macos")]
pub fn open_release_page(url: &str) {
    use std::process::Command;

    info!(url = url, "Opening release page in browser");

    if let Err(e) = Command::new("open").arg(url).spawn() {
        error!(error = ?e, "Failed to open release page");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn open_release_page(url: &str) {
    warn!(
        url = url,
        "Opening browser not implemented for this platform"
    );
}

/// Shows a system notification about the available update.
///
/// Uses macOS native notification system via `osascript`.
///
/// # Arguments
///
/// * `current` - The current installed version.
/// * `latest` - The latest available version.
#[cfg(target_os = "macos")]
pub fn show_update_notification(current: &str, latest: &str) {
    use std::process::Command;

    let title = "ExactoBar Update Available";
    let body = format!(
        "Version {} is available (you have {}). Click to download.",
        latest, current
    );

    let script = format!(
        r#"display notification "{}" with title "{}""#,
        body.replace('"', "\\\""),
        title
    );

    if let Err(e) = Command::new("osascript").args(["-e", &script]).spawn() {
        warn!(error = ?e, "Failed to show update notification");
    } else {
        debug!("Showed update notification");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn show_update_notification(current: &str, latest: &str) {
    info!(
        current = current,
        latest = latest,
        "Update available (notification not shown - not on macOS)"
    );
}

// ============================================================================
// Private Implementation
// ============================================================================

/// Internal implementation of the update check.
///
/// Uses `smol::unblock` to run the blocking HTTP request in a thread pool,
/// avoiding conflicts between smol (GPUI's runtime) and Tokio (reqwest's runtime).
async fn do_check_for_updates() -> UpdateCheckResult {
    info!("Checking for updates...");

    // Run the blocking HTTP request in a thread pool
    smol::unblock(|| do_check_for_updates_blocking()).await
}

/// Blocking implementation of the update check.
///
/// This runs in a thread pool via `smol::unblock`.
fn do_check_for_updates_blocking() -> UpdateCheckResult {
    // Build HTTP client with appropriate user agent
    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("ExactoBar/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            error!(error = ?e, "Failed to create HTTP client");
            return UpdateCheckResult::Error(format!("Failed to create HTTP client: {}", e));
        }
    };

    // Fetch latest release from GitHub API
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    debug!(url = url, "Fetching latest release info");

    let response = match client.get(&url).send() {
        Ok(r) => r,
        Err(e) => {
            error!(error = ?e, "Failed to fetch releases");
            return UpdateCheckResult::Error(format!("Failed to fetch releases: {}", e));
        }
    };

    // Check for successful response
    if !response.status().is_success() {
        let status = response.status();
        warn!(status = ?status, "GitHub API returned error status");
        return UpdateCheckResult::Error(format!("GitHub API returned status {}", status));
    }

    // Parse JSON response
    let release: serde_json::Value = match response.json() {
        Ok(j) => j,
        Err(e) => {
            error!(error = ?e, "Failed to parse release response");
            return UpdateCheckResult::Error(format!("Failed to parse response: {}", e));
        }
    };

    // Extract version from tag_name (e.g., "v0.2.0" -> "0.2.0")
    let tag_name = release["tag_name"].as_str().unwrap_or("");
    let latest_version = tag_name.trim_start_matches('v');

    debug!(
        current = CURRENT_VERSION,
        latest = latest_version,
        "Version comparison"
    );

    // Parse and compare versions using semver
    let current = match Version::parse(CURRENT_VERSION) {
        Ok(v) => v,
        Err(e) => {
            error!(error = ?e, version = CURRENT_VERSION, "Invalid current version");
            return UpdateCheckResult::Error("Invalid current version".to_string());
        }
    };

    let latest = match Version::parse(latest_version) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e, version = latest_version, "Invalid latest version");
            return UpdateCheckResult::Error(format!("Invalid latest version: {}", latest_version));
        }
    };

    // Compare versions - is there a newer one?
    if latest > current {
        // Find the macOS asset download URL from release assets
        let download_url = extract_macos_download_url(&release);

        info!(
            current = CURRENT_VERSION,
            latest = latest_version,
            "Update available!"
        );

        UpdateCheckResult::UpdateAvailable {
            current: CURRENT_VERSION.to_string(),
            latest: latest_version.to_string(),
            release_url: release["html_url"].as_str().unwrap_or("").to_string(),
            download_url,
            release_notes: release["body"].as_str().map(|s| s.to_string()),
        }
    } else {
        info!(version = CURRENT_VERSION, "Already on latest version");
        UpdateCheckResult::UpToDate
    }
}

/// Extracts the macOS download URL from release assets.
///
/// Looks for assets with names containing "macos", "darwin", or ending in ".dmg".
fn extract_macos_download_url(release: &serde_json::Value) -> Option<String> {
    release["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|a| {
                let name = a["name"].as_str().unwrap_or("").to_lowercase();
                #[allow(clippy::case_sensitive_file_extension_comparisons)]
                {
                    name.contains("macos") || name.contains("darwin") || name.ends_with(".dmg")
                }
            })
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .map(|s| s.to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_is_valid_semver() {
        assert!(Version::parse(CURRENT_VERSION).is_ok());
    }

    #[test]
    fn test_extract_macos_download_url() {
        let release = serde_json::json!({
            "assets": [
                {
                    "name": "exactobar-linux-x64.tar.gz",
                    "browser_download_url": "https://example.com/linux"
                },
                {
                    "name": "exactobar-macos-arm64.dmg",
                    "browser_download_url": "https://example.com/macos"
                }
            ]
        });

        let url = extract_macos_download_url(&release);
        assert_eq!(url, Some("https://example.com/macos".to_string()));
    }

    #[test]
    fn test_extract_macos_download_url_with_darwin() {
        let release = serde_json::json!({
            "assets": [
                {
                    "name": "exactobar-darwin-arm64.tar.gz",
                    "browser_download_url": "https://example.com/darwin"
                }
            ]
        });

        let url = extract_macos_download_url(&release);
        assert_eq!(url, Some("https://example.com/darwin".to_string()));
    }

    #[test]
    fn test_extract_macos_download_url_with_dmg() {
        let release = serde_json::json!({
            "assets": [
                {
                    "name": "ExactoBar.dmg",
                    "browser_download_url": "https://example.com/dmg"
                }
            ]
        });

        let url = extract_macos_download_url(&release);
        assert_eq!(url, Some("https://example.com/dmg".to_string()));
    }

    #[test]
    fn test_extract_macos_download_url_no_match() {
        let release = serde_json::json!({
            "assets": [
                {
                    "name": "exactobar-windows-x64.exe",
                    "browser_download_url": "https://example.com/windows"
                }
            ]
        });

        let url = extract_macos_download_url(&release);
        assert!(url.is_none());
    }
}
