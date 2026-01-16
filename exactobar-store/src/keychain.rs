//! Secure API key storage using the system keychain.
//!
//! This module provides synchronous access to the system's secure credential storage:
//! - macOS: Keychain Services
//! - Windows: Credential Manager
//! - Linux: Secret Service (GNOME Keyring, KDE Wallet)
//!
//! ## Usage
//!
//! ```ignore
//! use exactobar_store::keychain;
//!
//! // Store an API key
//! keychain::store_api_key("synthetic", "sk-xxxxx")?;
//!
//! // Retrieve an API key
//! if let Some(key) = keychain::get_api_key("synthetic") {
//!     println!("Got key: {}", &key[..8]);
//! }
//!
//! // Check if key exists
//! if keychain::has_api_key("synthetic") {
//!     println!("Key is configured!");
//! }
//!
//! // Delete an API key
//! keychain::delete_api_key("synthetic")?;
//! ```

use keyring::Entry;
use tracing::{debug, warn};

/// Service name prefix for `ExactoBar` credentials.
const SERVICE_PREFIX: &str = "ExactoBar";

/// Common provider names for API keys.
pub mod providers {
    /// Synthetic.new provider.
    pub const SYNTHETIC: &str = "synthetic";
    /// z.ai provider.
    pub const ZAI: &str = "zai";
    /// `OpenAI` Codex provider.
    pub const CODEX: &str = "codex";
    /// Google Gemini provider.
    pub const GEMINI: &str = "gemini";
}

/// Store an API key in the system keychain.
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "synthetic", "zai", "codex")
/// * `api_key` - The API key to store securely
///
/// # Errors
/// Returns an error string if the keychain operation fails.
///
/// # Example
/// ```ignore
/// keychain::store_api_key("synthetic", "sk-xxxxx")?;
/// ```
pub fn store_api_key(provider: &str, api_key: &str) -> Result<(), String> {
    let service = format!("{SERVICE_PREFIX}-{provider}");
    let entry = Entry::new(&service, "api_key")
        .map_err(|e| format!("Failed to create keychain entry: {e}"))?;

    entry
        .set_password(api_key)
        .map_err(|e| format!("Failed to store API key: {e}"))?;

    debug!(provider = provider, "API key stored in keychain");
    Ok(())
}

/// Retrieve an API key from the system keychain.
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "synthetic", "zai", "codex")
///
/// # Returns
/// * `Some(key)` - The API key if found and non-empty
/// * `None` - If no key is stored or the key is empty
///
/// # Example
/// ```ignore
/// if let Some(key) = keychain::get_api_key("synthetic") {
///     println!("Got key!");
/// }
/// ```
pub fn get_api_key(provider: &str) -> Option<String> {
    let service = format!("{SERVICE_PREFIX}-{provider}");
    let entry = Entry::new(&service, "api_key").ok()?;

    match entry.get_password() {
        Ok(key) if !key.is_empty() => {
            debug!(provider = provider, "API key retrieved from keychain");
            Some(key)
        }
        Ok(_) | Err(keyring::Error::NoEntry) => None, // Empty key treated as missing
        Err(e) => {
            warn!(provider = provider, error = %e, "Failed to retrieve API key");
            None
        }
    }
}

/// Delete an API key from the system keychain.
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "synthetic", "zai", "codex")
///
/// # Errors
/// Returns an error string if the deletion fails (ignores "not found" errors).
///
/// # Example
/// ```ignore
/// keychain::delete_api_key("synthetic")?;
/// ```
pub fn delete_api_key(provider: &str) -> Result<(), String> {
    let service = format!("{SERVICE_PREFIX}-{provider}");
    let entry = Entry::new(&service, "api_key")
        .map_err(|e| format!("Failed to create keychain entry: {e}"))?;

    match entry.delete_credential() {
        Ok(()) => {
            debug!(provider = provider, "API key deleted from keychain");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted, that's fine
        Err(e) => Err(format!("Failed to delete API key: {e}")),
    }
}

/// Check if an API key exists in the system keychain.
///
/// # Arguments
/// * `provider` - Provider identifier (e.g., "synthetic", "zai", "codex")
///
/// # Returns
/// `true` if a non-empty API key is stored, `false` otherwise.
///
/// # Example
/// ```ignore
/// if keychain::has_api_key("synthetic") {
///     println!("Synthetic API key is configured!");
/// }
/// ```
pub fn has_api_key(provider: &str) -> bool {
    get_api_key(provider).is_some()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_format() {
        // Verify service name follows expected format
        let service = format!("{}-{}", SERVICE_PREFIX, "synthetic");
        assert_eq!(service, "ExactoBar-synthetic");

        let service = format!("{}-{}", SERVICE_PREFIX, "codex");
        assert_eq!(service, "ExactoBar-codex");
    }

    #[test]
    fn test_provider_constants() {
        assert_eq!(providers::SYNTHETIC, "synthetic");
        assert_eq!(providers::ZAI, "zai");
        assert_eq!(providers::CODEX, "codex");
        assert_eq!(providers::GEMINI, "gemini");
    }

    // Note: Actual keychain operations require platform access and are typically
    // run as integration tests. These unit tests verify the string formatting.
}
