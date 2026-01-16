//! Kiro provider implementation.
//!
//! Kiro uses CLI-based usage: `kiro-cli /usage`

mod cli;
mod descriptor;
mod error;
mod fetcher;
pub(crate) mod parser;
mod strategies;

pub use cli::{KiroCliClient, KiroUsage, detect_version, ensure_logged_in};
pub use descriptor::kiro_descriptor;
pub use error::KiroError;
pub use fetcher::KiroUsageFetcher;
pub use strategies::KiroCliStrategy;
