use crate::models::{CommandSuggestion, SystemContext};
use anyhow::Result;
use async_trait::async_trait;

pub mod openrouter;

/// Trait for AI providers that can generate command suggestions
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generate a command suggestion based on a natural language query
    async fn generate_command(
        &self,
        query: &str,
        context: &SystemContext,
    ) -> Result<CommandSuggestion>;
}
