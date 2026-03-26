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

        let prompt = "Rate the person's sitting posture from 1 to 10. \
            1 = terrible posture (severe slouching, head far forward). \
            10 = perfect posture (straight back, shoulders aligned). \
            If no person is visible, reply 'N'. \
            Reply with ONLY a single number (1-10) or 'N'.";

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
    if let Some(content) = response_json["choices"][0]["message"]["content"].as_str() {
        return parse_posture_status(content);
    }

    log_error!("Could not parse API response: {:?}", response_json);
    anyhow::bail!("Could not parse API response");
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

    log_error!("Unexpected LLM response: {}", content);
    anyhow::bail!("Unexpected response: {}", content);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_status_accepts_valid_score() {
        assert_eq!(parse_posture_status("7").unwrap(), PostureStatus::Score(7));
    }

    #[test]
    fn parse_status_trims_whitespace() {
        assert_eq!(
            parse_posture_status(" 10 \n").unwrap(),
            PostureStatus::Score(10)
        );
    }

    #[test]
    fn parse_status_supports_no_person_marker() {
        assert_eq!(parse_posture_status("N").unwrap(), PostureStatus::NoPerson);
        assert_eq!(parse_posture_status("n").unwrap(), PostureStatus::NoPerson);
    }

    #[test]
    fn parse_status_rejects_invalid_values() {
        assert!(parse_posture_status("0").is_err());
        assert!(parse_posture_status("11").is_err());
        assert!(parse_posture_status("bad posture").is_err());
    }

    #[test]
    fn parse_api_response_extracts_nested_content() {
        let response_json = json!({
            "choices": [{
                "message": {
                    "content": "8"
                }
            }]
        });

        assert_eq!(
            parse_api_response(&response_json).unwrap(),
            PostureStatus::Score(8)
        );
    }

    #[test]
    fn parse_api_response_errors_when_content_missing() {
        let response_json = json!({"choices": []});
        assert!(parse_api_response(&response_json).is_err());
    }
}
