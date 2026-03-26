use crate::config::Config;
use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde_json::json;

pub enum PostureStatus {
    Score(u32),
    NoPerson,
}

pub struct PostureAnalyzer {
    client: Client,
    config: Config,
}

impl PostureAnalyzer {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn analyze(&self, image_data: &[u8]) -> Result<PostureStatus> {
        if self.config.api_key.is_empty() {
            anyhow::bail!("API key not configured");
        }

        let base64_image = base64::engine::general_purpose::STANDARD.encode(image_data);

        let prompt = "Rate the person's sitting posture from 1 to 10. \
            1 = terrible posture (severe slouching, head far forward). \
            10 = perfect posture (straight back, shoulders aligned). \
            If no person is visible, reply 'N'. \
            Reply with ONLY a single number (1-10) or 'N'.";

        let body = json!({
            "model": self.config.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", base64_image) } }
                ]
            }],
            "max_tokens": 5,
            "temperature": 0
        });

        let resp = self
            .client
            .post(&self.config.provider_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;

        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            let text = content.trim();
            if text.eq_ignore_ascii_case("n") {
                return Ok(PostureStatus::NoPerson);
            }
            if let Ok(score) = text.parse::<u32>() {
                if (1..=10).contains(&score) {
                    return Ok(PostureStatus::Score(score));
                }
            }
            anyhow::bail!("Unexpected response: {}", content);
        }

        anyhow::bail!("Could not parse API response");
    }
}
