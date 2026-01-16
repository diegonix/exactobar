//! Synthetic.new provider descriptor.

use exactobar_core::{IconStyle, ProviderBranding, ProviderColor, ProviderKind, ProviderMetadata};
use exactobar_fetch::{FetchContext, FetchPipeline, SourceMode};

use super::strategies::SyntheticApiStrategy;
use crate::descriptor::{CliConfig, FetchPlan, ProviderDescriptor, TokenCostConfig};

// ============================================================================
// Descriptor
// ============================================================================

/// Creates the Synthetic.new provider descriptor.
pub fn synthetic_descriptor() -> ProviderDescriptor {
    ProviderDescriptor {
        id: ProviderKind::Synthetic,
        metadata: synthetic_metadata(),
        branding: synthetic_branding(),
        token_cost: TokenCostConfig::default(),
        fetch_plan: synthetic_fetch_plan(),
        cli: synthetic_cli_config(),
    }
}

// ============================================================================
// Metadata
// ============================================================================

fn synthetic_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: ProviderKind::Synthetic,
        display_name: "Synthetic.new".to_string(),
        session_label: "Requests".to_string(),
        weekly_label: "Monthly".to_string(),
        opus_label: None,
        supports_opus: false,
        supports_credits: false,
        credits_hint: String::new(),
        toggle_title: "Show Synthetic.new usage".to_string(),
        cli_name: "synthetic".to_string(),
        default_enabled: false,
        is_primary_provider: false,
        uses_account_fallback: false,
        dashboard_url: Some("https://synthetic.new/dashboard".to_string()),
        subscription_dashboard_url: Some("https://synthetic.new/dashboard".to_string()),
        status_page_url: Some("https://status.synthetic.new".to_string()),
        status_link_url: Some("https://status.synthetic.new".to_string()),
    }
}

// ============================================================================
// Branding
// ============================================================================

fn synthetic_branding() -> ProviderBranding {
    ProviderBranding {
        icon_style: IconStyle::Synthetic,
        icon_resource_name: "icon_synthetic".to_string(),
        // Synthetic.new brand color - a nice teal/cyan
        color: ProviderColor::new(0.0, 0.8, 0.7),
    }
}

// ============================================================================
// Fetch Plan
// ============================================================================

fn synthetic_fetch_plan() -> FetchPlan {
    FetchPlan {
        source_modes: vec![SourceMode::ApiKey],
        build_pipeline: build_synthetic_pipeline,
    }
}

fn build_synthetic_pipeline(ctx: &FetchContext) -> FetchPipeline {
    let mut strategies: Vec<Box<dyn exactobar_fetch::FetchStrategy>> = Vec::new();

    if ctx.settings.source_mode.allows_api_key() {
        strategies.push(Box::new(SyntheticApiStrategy::new()));
    }

    FetchPipeline::with_strategies(strategies)
}

// ============================================================================
// CLI Config
// ============================================================================

fn synthetic_cli_config() -> CliConfig {
    CliConfig {
        name: "synthetic",
        aliases: &["syn"],
        version_args: &["--version"],
        usage_args: &["usage"],
    }
}
