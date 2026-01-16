//! Synthetic.new provider implementation.
//!
//! Synthetic.new is a third-party LLM provider offering open-source models
//! through flat-rate subscriptions with simple API key authentication.

mod api;
mod descriptor;
mod error;
mod strategies;

pub use api::{SyntheticApiClient, SyntheticQuotaResponse};
pub use descriptor::synthetic_descriptor;
pub use error::SyntheticError;
pub use strategies::SyntheticApiStrategy;
