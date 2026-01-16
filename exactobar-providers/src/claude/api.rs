//! Claude API client for OAuth-based usage fetching.
//!
//! This module provides a client for the Anthropic API to fetch usage data.
//!
//! # API Endpoint
//!
//! ```text
//! GET https://api.anthropic.com/api/oauth/usage
//! Authorization: Bearer sk-ant-oat01-YOUR_TOKEN
//! anthropic-beta: oauth-2025-04-20
//! ```
//!
//! # Response Format
//!
//! ```json
//! {
//!   "five_hour": { "utilization": 6.0, "resets_at": "2025-11-04T04:59:59.943648+00:00" },
//!   "seven_day": { "utilization": 35.0, "resets_at": "2025-11-06T03:59:59.943679+00:00" },
//!   "seven_day_opus": { "utilization": 0.0, "resets_at": null }
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{debug, info, instrument, warn};

use super::error::ClaudeError;
use super::oauth::ClaudeOAuthCredentials;

// ============================================================================
// Constants
// ============================================================================

/// Base URL for Anthropic API.
pub const API_BASE_URL: &str = "https://api.anthropic.com";

/// OAuth usage endpoint.
pub const USAGE_ENDPOINT: &str = "/api/oauth/usage";

/// The ESSENTIAL beta header for OAuth usage API.
pub const ANTHROPIC_BETA_HEADER: &str = "oauth-2025-04-20";

// ============================================================================
// OAuth Usage API Response Structures
// ============================================================================

/// Response from the OAuth usage API.
/// Note: Uses snake_case field names matching the actual API response.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthUsageResponse {
    /// 5-hour usage window.
    pub five_hour: Option<OAuthUsageWindow>,
    /// 7-day usage window (all models).
    pub seven_day: Option<OAuthUsageWindow>,
    /// 7-day Opus usage window.
    pub seven_day_opus: Option<OAuthUsageWindow>,
    /// 7-day OAuth apps usage window (optional).
    #[allow(dead_code)]
    pub seven_day_oauth_apps: Option<OAuthUsageWindow>,
}

/// Individual usage window from OAuth API.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthUsageWindow {
    /// Utilization percentage (0-100).
    pub utilization: f64,
    /// When this window resets (ISO 8601).
    pub resets_at: Option<String>,
}

#[allow(dead_code)]
impl OAuthUsageWindow {
    /// Get the used percentage.
    pub fn get_used_percent(&self) -> f64 {
        self.utilization
    }

    /// Parse the reset timestamp.
    pub fn get_resets_at(&self) -> Option<DateTime<Utc>> {
        self.resets_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
    }
}

impl OAuthUsageResponse {
    /// Convert to UsageApiResponse for compatibility.
    pub fn into_usage_api_response(self) -> UsageApiResponse {
        UsageApiResponse {
            five_hour: self.five_hour.map(|w| UsageWindow {
                utilization: w.utilization,
                resets_at: w.resets_at,
                remaining: None,
                used_percent: None,
            }),
            seven_day: self.seven_day.map(|w| UsageWindow {
                utilization: w.utilization,
                resets_at: w.resets_at,
                remaining: None,
                used_percent: None,
            }),
            // Map seven_day_opus to seven_day_sonnet for compatibility
            seven_day_sonnet: self.seven_day_opus.map(|w| UsageWindow {
                utilization: w.utilization,
                resets_at: w.resets_at,
                remaining: None,
                used_percent: None,
            }),
            extra_usage: None,
            account: None,
        }
    }
}

// ============================================================================
// Unified API Response Structures (for internal compatibility)
// ============================================================================

/// Unified response for internal usage.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageApiResponse {
    /// 5-hour usage window.
    pub five_hour: Option<UsageWindow>,
    /// 7-day usage window (all models).
    pub seven_day: Option<UsageWindow>,
    /// 7-day Sonnet/Opus usage window.
    pub seven_day_sonnet: Option<UsageWindow>,
    /// Extra usage/credits info.
    pub extra_usage: Option<ExtraUsage>,
    /// Account info.
    pub account: Option<AccountInfo>,
}

/// Individual usage window.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    /// Utilization percentage (0-100).
    pub utilization: f64,
    /// When this window resets (ISO 8601).
    pub resets_at: Option<String>,
    /// Remaining percentage (alternative field).
    pub remaining: Option<f64>,
    /// Used percentage (alternative field).
    pub used_percent: Option<f64>,
}

impl UsageWindow {
    /// Get the used percentage, handling various field names.
    pub fn get_used_percent(&self) -> f64 {
        // utilization is the "used" percentage
        if self.utilization > 0.0 {
            return self.utilization;
        }
        if let Some(used) = self.used_percent {
            return used;
        }
        if let Some(remaining) = self.remaining {
            return 100.0 - remaining;
        }
        0.0
    }

    /// Parse the reset timestamp.
    pub fn get_resets_at(&self) -> Option<DateTime<Utc>> {
        self.resets_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
    }
}

/// Extra usage/credits information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraUsage {
    /// Whether extra usage is enabled.
    pub is_enabled: Option<bool>,
    /// Credits used this month.
    pub used_credits: Option<f64>,
    /// Monthly credit limit.
    pub monthly_limit: Option<f64>,
    /// Currency (e.g., "USD").
    pub currency: Option<String>,
}

/// Account information from API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Account email.
    pub email: Option<String>,
    /// Account name.
    pub name: Option<String>,
    /// Plan name.
    pub plan: Option<String>,
    /// Organization name.
    pub organization: Option<String>,
}

// ============================================================================
// API Client
// ============================================================================

/// Claude API client for fetching usage data.
#[derive(Debug, Clone)]
pub struct ClaudeApiClient {
    base_url: String,
}

impl Default for ClaudeApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeApiClient {
    /// Create a new API client.
    pub fn new() -> Self {
        Self {
            base_url: API_BASE_URL.to_string(),
        }
    }

    /// Create a client with a custom base URL.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Fetch usage data using OAuth credentials.
    #[instrument(skip(self, credentials))]
    pub async fn fetch_usage(
        &self,
        credentials: &ClaudeOAuthCredentials,
    ) -> Result<UsageApiResponse, ClaudeError> {
        let url = format!("{}{}", self.base_url, USAGE_ENDPOINT);

        debug!(url = %url, "Fetching Claude usage via OAuth");

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| ClaudeError::HttpError(e.to_string()))?;

        let response = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", credentials.access_token),
            )
            .header("anthropic-beta", ANTHROPIC_BETA_HEADER) // ESSENTIAL!
            .header("Content-Type", "application/json")
            .header("User-Agent", "claude-code/2.0.32")
            .header("Accept", "application/json, text/plain, */*")
            .send()
            .await
            .map_err(|e| ClaudeError::HttpError(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ClaudeError::AuthenticationFailed(
                "OAuth token rejected".to_string(),
            ));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(ClaudeError::MissingScope("user:profile".to_string()));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "API request failed");
            return Err(ClaudeError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ClaudeError::HttpError(e.to_string()))?;

        debug!(response_length = body.len(), "Got API response");
        info!(
            "Raw OAuth usage response: {}",
            &body[..body.len().min(2000)]
        );

        // Parse OAuth usage response
        let oauth_response: OAuthUsageResponse = serde_json::from_str(&body).map_err(|e| {
            ClaudeError::ParseError(format!(
                "Failed to parse OAuth response: {} - body: {}",
                e,
                &body[..body.len().min(500)]
            ))
        })?;

        info!(
            "Parsed OAuth usage: five_hour={:?}, seven_day={:?}, seven_day_opus={:?}",
            oauth_response.five_hour, oauth_response.seven_day, oauth_response.seven_day_opus
        );

        // Convert to UsageApiResponse for internal compatibility
        Ok(oauth_response.into_usage_api_response())
    }

    /// Fetch usage using the access token directly.
    #[instrument(skip(self, access_token))]
    pub async fn fetch_usage_with_token(
        &self,
        access_token: &str,
    ) -> Result<UsageApiResponse, ClaudeError> {
        let url = format!("{}{}", self.base_url, USAGE_ENDPOINT);

        debug!(url = %url, "Fetching usage from API with token");

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| ClaudeError::HttpError(e.to_string()))?;

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("anthropic-beta", ANTHROPIC_BETA_HEADER) // ESSENTIAL!
            .header("Content-Type", "application/json")
            .header("User-Agent", "claude-code/2.0.32")
            .header("Accept", "application/json, text/plain, */*")
            .send()
            .await
            .map_err(|e| ClaudeError::HttpError(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ClaudeError::AuthenticationFailed(
                "Token rejected".to_string(),
            ));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClaudeError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ClaudeError::HttpError(e.to_string()))?;

        debug!(
            "Raw OAuth usage response: {}",
            &body[..body.len().min(2000)]
        );

        // Parse OAuth usage response
        let oauth_response: OAuthUsageResponse =
            serde_json::from_str(&body).map_err(|e| ClaudeError::ParseError(e.to_string()))?;

        Ok(oauth_response.into_usage_api_response())
    }
}

// ============================================================================
// Conversion to Core Types
// ============================================================================

impl UsageApiResponse {
    /// Convert to a UsageSnapshot.
    pub fn to_snapshot(&self) -> exactobar_core::UsageSnapshot {
        use exactobar_core::{FetchSource, LoginMethod, ProviderIdentity, ProviderKind};

        let mut snapshot = exactobar_core::UsageSnapshot::new();
        snapshot.fetch_source = FetchSource::OAuth;

        // Primary = 5-hour window
        if let Some(ref window) = self.five_hour {
            snapshot.primary = Some(exactobar_core::UsageWindow {
                used_percent: window.get_used_percent(),
                window_minutes: Some(300), // 5 hours
                resets_at: window.get_resets_at(),
                reset_description: None,
            });
        }

        // Secondary = 7-day window (all models)
        if let Some(ref window) = self.seven_day {
            snapshot.secondary = Some(exactobar_core::UsageWindow {
                used_percent: window.get_used_percent(),
                window_minutes: Some(10080), // 7 days
                resets_at: window.get_resets_at(),
                reset_description: None,
            });
        }

        // Tertiary = 7-day Opus window (mapped from seven_day_sonnet for compat)
        if let Some(ref window) = self.seven_day_sonnet {
            snapshot.tertiary = Some(exactobar_core::UsageWindow {
                used_percent: window.get_used_percent(),
                window_minutes: Some(10080), // 7 days
                resets_at: window.get_resets_at(),
                reset_description: None,
            });
        }

        // Account identity
        if let Some(ref account) = self.account {
            let mut identity = ProviderIdentity::new(ProviderKind::Claude);
            identity.account_email = account.email.clone();
            identity.plan_name = account.plan.clone();
            identity.account_organization = account.organization.clone();
            identity.login_method = Some(LoginMethod::OAuth);
            snapshot.identity = Some(identity);
        }

        snapshot
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_oauth_usage_response() {
        let json = r#"{
            "five_hour": {
                "utilization": 6.0,
                "resets_at": "2025-11-04T04:59:59.943648+00:00"
            },
            "seven_day": {
                "utilization": 35.0,
                "resets_at": "2025-11-06T03:59:59.943679+00:00"
            },
            "seven_day_opus": {
                "utilization": 0.0,
                "resets_at": null
            }
        }"#;

        let response: OAuthUsageResponse = serde_json::from_str(json).unwrap();

        let five_hour = response.five_hour.as_ref().unwrap();
        assert!((five_hour.utilization - 6.0).abs() < 0.01);
        assert!(five_hour.get_resets_at().is_some());

        let seven_day = response.seven_day.as_ref().unwrap();
        assert!((seven_day.utilization - 35.0).abs() < 0.01);
        assert!(seven_day.get_resets_at().is_some());

        let opus = response.seven_day_opus.as_ref().unwrap();
        assert!((opus.utilization - 0.0).abs() < 0.01);
        assert!(opus.resets_at.is_none());
    }

    #[test]
    fn test_oauth_to_usage_api_response() {
        let oauth_response = OAuthUsageResponse {
            five_hour: Some(OAuthUsageWindow {
                utilization: 25.0,
                resets_at: Some("2025-01-01T12:00:00Z".to_string()),
            }),
            seven_day: Some(OAuthUsageWindow {
                utilization: 50.0,
                resets_at: None,
            }),
            seven_day_opus: Some(OAuthUsageWindow {
                utilization: 10.0,
                resets_at: None,
            }),
            seven_day_oauth_apps: None,
        };

        let response = oauth_response.into_usage_api_response();

        assert!(response.five_hour.is_some());
        assert!((response.five_hour.as_ref().unwrap().utilization - 25.0).abs() < 0.01);

        assert!(response.seven_day.is_some());
        assert!((response.seven_day.as_ref().unwrap().utilization - 50.0).abs() < 0.01);

        assert!(response.seven_day_sonnet.is_some());
        assert!((response.seven_day_sonnet.as_ref().unwrap().utilization - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_usage_window_get_used_percent() {
        // Test utilization field
        let window = UsageWindow {
            utilization: 25.0,
            resets_at: None,
            remaining: None,
            used_percent: None,
        };
        assert!((window.get_used_percent() - 25.0).abs() < 0.01);

        // Test remaining field (75% remaining = 25% used)
        let window = UsageWindow {
            utilization: 0.0,
            resets_at: None,
            remaining: Some(75.0),
            used_percent: None,
        };
        assert!((window.get_used_percent() - 25.0).abs() < 0.01);

        // Test used_percent field
        let window = UsageWindow {
            utilization: 0.0,
            resets_at: None,
            remaining: None,
            used_percent: Some(30.0),
        };
        assert!((window.get_used_percent() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_to_snapshot() {
        let response = UsageApiResponse {
            five_hour: Some(UsageWindow {
                utilization: 25.0,
                resets_at: Some("2025-01-01T12:00:00Z".to_string()),
                remaining: None,
                used_percent: None,
            }),
            seven_day: Some(UsageWindow {
                utilization: 45.0,
                resets_at: None,
                remaining: None,
                used_percent: None,
            }),
            seven_day_sonnet: None,
            extra_usage: None,
            account: Some(AccountInfo {
                email: Some("test@example.com".to_string()),
                name: None,
                plan: Some("pro".to_string()),
                organization: None,
            }),
        };

        let snapshot = response.to_snapshot();

        assert!(snapshot.primary.is_some());
        assert!((snapshot.primary.as_ref().unwrap().used_percent - 25.0).abs() < 0.01);

        assert!(snapshot.secondary.is_some());
        assert!((snapshot.secondary.as_ref().unwrap().used_percent - 45.0).abs() < 0.01);

        assert!(snapshot.tertiary.is_none());

        assert!(snapshot.identity.is_some());
        assert_eq!(
            snapshot.identity.as_ref().unwrap().account_email,
            Some("test@example.com".to_string())
        );
    }

    #[test]
    fn test_client_creation() {
        let client = ClaudeApiClient::new();
        assert_eq!(client.base_url, "https://api.anthropic.com");

        let custom = ClaudeApiClient::with_base_url("https://custom.api.com");
        assert_eq!(custom.base_url, "https://custom.api.com");
    }
}
