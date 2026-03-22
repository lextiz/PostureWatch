use crate::config::Config;
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
        // Prompt that handles no-person cases
        let prompt = "You are a posture expert. Look at this image and analyze what you see.
- If you can see a person's face or upper body, analyze their sitting posture. Is their back straight? Are shoulders level? Reply with 'Good' if posture is acceptable, 'Bad' if they are slouching or have poor posture.
- If you cannot see a clear human face or body in the frame (empty room, no person, camera pointed away, person too far, etc.), reply with ONLY 'NoPerson'.
Reply with ONLY one word: 'Good', 'Bad', or 'NoPerson'.";

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
            println!("API Response: {}", content);
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
