use crate::config::Config;
use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde_json::json;

pub enum PostureStatus {
    Good,
    Bad,
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
            anyhow::bail!("API key not configured. Please restart and enter your API key.");
        }

        let base64_image = base64::engine::general_purpose::STANDARD.encode(image_data);
        let prompt = "Analyze the posture of the person in this image. Is their back straight and shoulders relaxed? Answer strictly 'Good' or 'Bad'.";

        let body = json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/jpeg;base64,{}", base64_image)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 10
        });

        let resp = self
            .client
            .post(&self.config.provider_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, body_text);
        }

        let json = resp.json::<serde_json::Value>().await?;
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            let text = content.to_lowercase();
            if text.contains("good") {
                return Ok(PostureStatus::Good);
            } else if text.contains("bad") {
                return Ok(PostureStatus::Bad);
            }
        }

        anyhow::bail!("Could not parse API response");
    }
}
