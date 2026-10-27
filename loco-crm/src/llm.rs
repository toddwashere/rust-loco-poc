//! Minimal LLM client. Anthropic has no official Rust SDK, so this calls the
//! Messages API over HTTP. Without an API key it falls back to a
//! deterministic mock so the app (and tests) run offline.
use std::{sync::LazyLock, time::Instant};

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::settings::Settings;

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_default()
});

#[derive(Debug, Clone, Serialize)]
pub struct Completion {
    pub text: String,
    pub model: String,
    pub mock: bool,
    pub elapsed_ms: u128,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

/// Run a single-turn completion.
///
/// # Errors
/// On transport errors or a non-2xx response from the provider.
pub async fn complete(ctx: &AppContext, system: &str, prompt: &str) -> Result<Completion> {
    let settings = Settings::from_ctx(ctx)?.ai;
    let start = Instant::now();

    if settings.anthropic_api_key.is_empty() {
        return Ok(Completion {
            text: mock_reply(system, prompt),
            model: "mock".to_string(),
            mock: true,
            elapsed_ms: start.elapsed().as_millis(),
        });
    }

    let res = HTTP
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &settings.anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": settings.model,
            "max_tokens": 600,
            "system": system,
            "messages": [{ "role": "user", "content": prompt }],
        }))
        .send()
        .await
        .map_err(|e| Error::string(&format!("LLM request failed: {e}")))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(Error::string(&format!("LLM returned {status}: {body}")));
    }

    let body: MessagesResponse = res
        .json()
        .await
        .map_err(|e| Error::string(&format!("LLM response was not understood: {e}")))?;
    let text = body
        .content
        .into_iter()
        .filter(|b| b.kind == "text")
        .map(|b| b.text)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(Completion {
        text,
        model: settings.model,
        mock: false,
        elapsed_ms: start.elapsed().as_millis(),
    })
}

fn mock_reply(system: &str, prompt: &str) -> String {
    let notes = prompt.lines().filter(|l| l.starts_with("- [")).count();
    let who = prompt
        .lines()
        .find_map(|l| l.strip_prefix("Contact: "))
        .unwrap_or("this contact");
    if system.contains("email") {
        format!(
            "[mock] Hi {who},\n\nThanks for the time recently — following up on our conversation. \
             Would a 20-minute call next week work to go over next steps?\n\nBest,\nThe team"
        )
    } else {
        format!(
            "[mock] {who} has {notes} note(s) on file. Set ANTHROPIC_API_KEY to get a real summary."
        )
    }
}
