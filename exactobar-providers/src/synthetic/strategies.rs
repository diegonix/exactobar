//! Synthetic.new fetch strategies.

use async_trait::async_trait;
#[allow(unused_imports)]
use exactobar_core::UsageSnapshot;
use exactobar_fetch::{FetchContext, FetchError, FetchKind, FetchResult, FetchStrategy};
use tracing::{debug, instrument};

use super::api::SyntheticApiClient;

// ============================================================================
// API Key Strategy
// ============================================================================

/// API key strategy for Synthetic.new.
///
/// Uses the SYNTHETIC_API_KEY environment variable to authenticate.
pub struct SyntheticApiStrategy;

impl SyntheticApiStrategy {
    /// Creates a new strategy.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SyntheticApiStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for SyntheticApiStrategy {
    fn id(&self) -> &str {
        "synthetic.api"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiKey
    }

    #[instrument(skip(self, _ctx))]
    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        SyntheticApiClient::get_api_key().is_ok()
    }

    #[instrument(skip(self, _ctx))]
    async fn fetch(&self, _ctx: &FetchContext) -> Result<FetchResult, FetchError> {
        debug!("Fetching Synthetic.new usage via API key");

        let api_key = SyntheticApiClient::get_api_key()
            .map_err(|e| FetchError::AuthenticationFailed(e.to_string()))?;

        let client = SyntheticApiClient::new();
        let response = client
            .fetch_quota(&api_key)
            .await
            .map_err(|e| FetchError::InvalidResponse(e.to_string()))?;

        debug!("Synthetic.new quota fetched successfully");
        let snapshot = response.to_snapshot();

        Ok(FetchResult::new(snapshot, self.id(), self.kind()))
    }

    fn priority(&self) -> u32 {
        60 // API Key priority
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_strategy() {
        let s = SyntheticApiStrategy::new();
        assert_eq!(s.id(), "synthetic.api");
        assert_eq!(s.kind(), FetchKind::ApiKey);
        assert_eq!(s.priority(), 60);
    }

    #[test]
    fn test_api_strategy_default() {
        let s = SyntheticApiStrategy;
        assert_eq!(s.id(), "synthetic.api");
    }
}
