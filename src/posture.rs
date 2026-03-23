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
            Strictness::High => "Be very strict. Even minor slouching, rounded shoulders, or leaning to one side is Bad.",
            Strictness::Medium => "Be moderately strict. Clear slouching, hunched back, or significantly uneven shoulders is Bad. Minor posture variations are acceptable.",
            Strictness::Low => "Be lenient. Only flag truly poor posture - severe slouching, chin on chest, or extreme forward lean. Slight imperfections are OK.",
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
