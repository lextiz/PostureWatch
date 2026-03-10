use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

pub enum PostureStatus {
    Good,
    Bad,
    Unknown,
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
        // Privacy mode or empty api key triggers local fallback immediately
        if self.config.privacy_mode || self.config.api_key.is_empty() {
            return self.local_fallback(image_data);
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

        match self
            .client
            .post(&self.config.provider_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                        let text = content.to_lowercase();
                        if text.contains("good") {
                            return Ok(PostureStatus::Good);
                        } else if text.contains("bad") {
                            return Ok(PostureStatus::Bad);
                        }
                    }
                }
            }
            Err(_) => {
                // Network error, drop to fallback
                return self.local_fallback(image_data);
            }
        }

        self.local_fallback(image_data)
    }

    fn local_fallback(&self, _image_data: &[u8]) -> Result<PostureStatus> {
        // A placeholder heuristic for local fallback.
        // In a real application, we might use a lightweight local model like ONNX or tflite,
        // or simple OpenCV contour analysis. Here we just assume it's Good if it reaches here,
        // or randomly determine for the sake of the requirement.
        // For production, if there's no model available, we can't reliably determine posture locally
        // without heavy dependencies. We'll return Unknown or rely on a simple time-based heuristic.

        Ok(PostureStatus::Unknown)
    }
}
