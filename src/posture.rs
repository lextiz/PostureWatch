use crate::config::Config;
use crate::log_error;
use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde_json::json;

#[derive(Debug, PartialEq, Eq)]
pub enum PostureStatus {
    Score(u32),
    NoPerson,
}

pub struct PostureAnalyzer {
    client: Client,
}

impl PostureAnalyzer {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn analyze(&self, image_data: &[u8], config: &Config) -> Result<PostureStatus> {
        if config.api_key.is_empty() {
            log_error!("API key not configured");
            anyhow::bail!("API key not configured");
        }

        let base64_image = base64::engine::general_purpose::STANDARD.encode(image_data);

        let prompt = if config.llm_prompt.trim().is_empty() {
            Config::default().llm_prompt
        } else {
            config.llm_prompt.clone()
        };

        // Use max_completion_tokens for newer models (gpt-4o, gpt-5.x)
        // Fall back to max_tokens for older models
        let body = json!({
            "model": config.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", base64_image) } }
                ]
            }],
            "max_completion_tokens": 10,
            "temperature": 0
        });

        let resp = self
            .client
            .post(&config.provider_endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            log_error!("API error {}: {}", status, text);
            anyhow::bail!("API error {}: {}", status, text);
        }

        let response_json: serde_json::Value = resp.json().await?;
        parse_api_response(&response_json)
    }
}

fn parse_api_response(response_json: &serde_json::Value) -> Result<PostureStatus> {
    if let Some(content) = extract_content_text(&response_json["choices"][0]["message"]["content"])
    {
        return parse_posture_status(&content);
    }

    log_error!("Could not parse API response: {:?}", response_json);
    anyhow::bail!("Could not parse API response");
}

fn extract_content_text(content: &serde_json::Value) -> Option<String> {
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    let parts = content.as_array()?;
    let mut merged = String::new();

    for part in parts {
        if part["type"].as_str() == Some("text") {
            if let Some(text) = part["text"].as_str() {
                if !merged.is_empty() {
                    merged.push('\n');
                }
                merged.push_str(text);
            }
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn parse_posture_status(content: &str) -> Result<PostureStatus> {
    let text = content.trim();

    if text.eq_ignore_ascii_case("n") {
        return Ok(PostureStatus::NoPerson);
    }

    if let Ok(score) = text.parse::<u32>() {
        if (1..=10).contains(&score) {
            return Ok(PostureStatus::Score(score));
        }
    }

    if let Some(score) = extract_score_token(text) {
        return Ok(PostureStatus::Score(score));
    }

    if has_no_person_token(text) {
        return Ok(PostureStatus::NoPerson);
    }

    log_error!("Unexpected LLM response: {}", content);
    anyhow::bail!("Unexpected response: {}", content);
}

fn extract_score_token(text: &str) -> Option<u32> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .find_map(|token| match token.parse::<u32>() {
            Ok(score) if (1..=10).contains(&score) => Some(score),
            _ => None,
        })
}

fn has_no_person_token(text: &str) -> bool {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token.eq_ignore_ascii_case("n"))
}

#[cfg(test)]
#[path = "tests/posture_tests.rs"]
mod tests;
