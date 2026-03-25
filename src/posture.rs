use crate::config::Config;
use crate::posture_monitor::Strictness;
use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde_json::json;

pub enum PostureStatus {
    Good,
    Bad,
    NoPerson, // No human detected in frame
}

pub struct PostureAnalyzer {
    client: Client,
    config: Config,
    strictness: Strictness,
}

impl PostureAnalyzer {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config: config.clone(),
            strictness: Strictness::from_str(&config.strictness),
        }
    }

    pub async fn analyze(&self, image_data: &[u8]) -> Result<PostureStatus> {
        if self.config.api_key.is_empty() {
            anyhow::bail!("API key not configured. Please restart and enter your API key.");
        }

        let base64_image = base64::engine::general_purpose::STANDARD.encode(image_data);

        // Strictness-aware prompt
        let strictness_guidance = match self.strictness {
            Strictness::High => {
                "Be somewhat strict. Only flag clear problems: forward head (chin significantly forward), \
clearly rounded shoulders, or sustained severe slouching. Don't flag minor variations."
            }
            Strictness::Medium => {
                "Be balanced. Only flag clear bad posture: forward head position, visibly rounded shoulders, \
or clearly hunched back. Normal sitting variations and brief moments are OK. When in doubt, say Good."
            }
            Strictness::Low => {
                "Be very lenient. Only flag severe, sustained bad posture - chin on chest, extreme forward lean, \
or very obvious slouching. Minor slouching, shifting, or normal posture variations are Good. Be conservative."
            }
        };

        let prompt = format!("You are a posture expert. Your task is to analyze the person's sitting posture.

Guidelines:
- {}
- If you cannot see a clear human face or upper body (empty room, no person, camera pointed away, person too far, blurred, etc.), reply with ONLY 'NoPerson'.

Output: Reply with ONLY one word: 'Good', 'Bad', or 'NoPerson'.", strictness_guidance);

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

        // Debug: print the response
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            let text = content.to_lowercase().trim().to_string();
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
