//! Reusable UI components.

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod provider_card;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod provider_icon;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod spinner;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod toggle;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod usage_bar;

#[allow(unused_imports)]
pub use provider_card::ProviderCard;
pub use provider_icon::ProviderIcon;
pub use spinner::Spinner;
pub use toggle::Toggle;
pub use usage_bar::UsageBar;
