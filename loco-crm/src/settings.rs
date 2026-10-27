//! Typed view of the `settings:` block in `config/<env>.yaml`.
use loco_rs::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AiSettings {
    #[serde(default)]
    pub anthropic_api_key: String,
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_model() -> String {
    "claude-haiku-5-5".to_string()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default = "default_notify")]
    pub notify_email: String,
}

fn default_notify() -> String {
    "sales@example.com".to_string()
}

impl Settings {
    /// # Errors
    /// When the `settings:` block does not match this shape.
    pub fn from_ctx(ctx: &AppContext) -> Result<Self> {
        match &ctx.config.settings {
            Some(value) => Ok(serde_json::from_value(value.clone())?),
            None => Ok(Self::default()),
        }
    }
}
