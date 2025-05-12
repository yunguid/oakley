//! Flash-card generation via LLM prompt - stub implementation.

use anyhow::Result;
use serde::Deserialize;
use tracing::{info, error};

/// Result fields coming back from the LLM
#[derive(Debug, Deserialize)]
pub struct CardFields {
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

// Mock llama_cpp namespace
#[cfg(feature = "full")]
mod llama_cpp {
    pub struct Params;
    
    impl Default for Params {
        fn default() -> Self {
            Self {}
        }
    }
    
    pub fn generate(_prompt: &str, _params: Params) -> Result<String, Box<dyn std::error::Error>> {
        // Mock implementation returns a fixed response
        Ok(r#"{"front":"What is the capital of France?","back":"Paris","tags":["geography","europe"]}"#.into())
    }
}

#[cfg(feature = "full")]
use reqwest::Client;
#[cfg(feature = "full")]
use serde_json::json;

/// Generate card JSON from plain text.
pub async fn gen_card(text: &str) -> Result<CardFields> {
    #[cfg(feature = "full")]
    {
        // Include the word "json" in the input text to satisfy API requirements
        let enhanced_text = format!("Create JSON flashcard from this text: {}", text);
        let instructions = "You are an expert pedagogue. For the given text create a concise flashcard JSON with keys front, back, tags (array of strings). Only output valid, structured JSON without any additional text.";
        
        info!("Enhanced input with JSON request: '{}'", enhanced_text);
        
        let body = json!({
            "model": "gpt-4o-mini", // cheaper default; change as desired
            "instructions": instructions,
            "input": enhanced_text,
            "temperature": 0.4,
            "max_output_tokens": 256,
            "text": { "format": { "type": "json_object" } }
        });

        // Get API key
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) => {
                info!("Found OpenAI API key (length: {})", key.len());
                key
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Environment variable OPENAI_API_KEY not set"));
            }
        };
        
        info!("Making OpenAI API request with model: {}", body["model"]);
        let client = Client::new();
        let response = client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            error!("OpenAI API error: Status={}, Body={}", status, error_body);
            return Err(anyhow::anyhow!("OpenAI API error: Status={}, Body={}", status, error_body));
        }

        info!("OpenAI API request successful");
        let resp: serde_json::Value = response.json().await?;

        // Extract the assistant text content.
        let content = resp
            .get("output")
            .and_then(|o| o.get(0))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Unexpected response structure from OpenAI"))?;

        Ok(serde_json::from_str(content)?)
    }

    #[cfg(not(feature = "full"))]
    {
        // Fallback stub.
        Ok(CardFields {
            front: format!("What is the gist of: {}?", text.lines().next().unwrap_or("text")),
            back: "stub answer".into(),
            tags: vec!["stub".into()],
        })
    }
}

/// Generate card JSON from a screenshot PNG/JPEG image.
pub async fn gen_card_from_image(image_bytes: &[u8]) -> Result<CardFields> {
    #[cfg(feature = "full")]
    {
        use base64::{engine::general_purpose, Engine as _};
        use reqwest::Client;
        use serde_json::json;

        // Encode bytes as data URL
        let b64 = general_purpose::STANDARD.encode(image_bytes);
        let data_url = format!("data:image/png;base64,{}", b64);

        let instructions = "You are an expert pedagogue. For the given image create a concise flashcard JSON with keys front, back, tags (array of strings). Only output valid, structured JSON without any additional text.";

        let body = json!({
            "model": "gpt-4o-mini",
            "instructions": instructions,
            "input": [{
                "role": "user",
                "content": [
                    {"type": "input_image", "image_url": data_url}
                ]
            }],
            "temperature": 0.4,
            "max_output_tokens": 256,
            "text": { "format": { "type": "json_object" } }
        });

        // Get API key
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("Environment variable OPENAI_API_KEY not set"))?;

        info!("Sending vision request ({} bytes image)", image_bytes.len());

        let client = Client::new();
        let response = client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let err = response.text().await?;
            error!(?status, ?err, "OpenAI vision API error");
            return Err(anyhow::anyhow!("OpenAI vision API error: {status} {err}"));
        }

        let resp: serde_json::Value = response.json().await?;

        let content = resp
            .get("output")
            .and_then(|o| o.get(0))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Unexpected response structure from OpenAI vision"))?;

        Ok(serde_json::from_str(content)?)
    }

    #[cfg(not(feature = "full"))]
    {
        Ok(CardFields {
            front: "stub front from image".into(),
            back: "stub back".into(),
            tags: vec!["stub".into()],
        })
    }
} 