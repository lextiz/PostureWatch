use crate::config::Config;
use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde_json::json;

pub enum PostureStatus {
    Good,
    Bad,
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

    fn strictness_guidance(&self) -> &'static str {
        match self.config.strictness.to_lowercase().as_str() {
            "high" => "Be strict. Flag clear problems: forward head, rounded shoulders, or slouching.",
            "low" => "Be lenient. Only flag severe bad posture - extreme forward lean or obvious slouching.",
            _ => "Be balanced. Flag clear bad posture. When in doubt, say Good.",
        }
    }

    pub async fn analyze(&self, image_data: &[u8]) -> Result<PostureStatus> {
        if self.config.api_key.is_empty() {
            anyhow::bail!("API key not configured");
        }

        let base64_image = base64::engine::general_purpose::STANDARD.encode(image_data);

        let prompt = format!(
            "Analyze sitting posture. {}. If no person visible, reply 'NoPerson'. \
            Reply ONLY: 'Good', 'Bad', or 'NoPerson'.",
            self.strictness_guidance()
        );

        let body = json!({
            "model": self.config.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", base64_image) } }
                ]
            }],
            "max_tokens": 10,
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
            let text = content.to_lowercase();
            if text.starts_with("good") {
                return Ok(PostureStatus::Good);
            } else if text.starts_with("bad") {
                return Ok(PostureStatus::Bad);
            } else if text.starts_with("noperson") {
                return Ok(PostureStatus::NoPerson);
            }
            anyhow::bail!("Unexpected response: {}", content);
        }

        anyhow::bail!("Could not parse API response");
    }
}
