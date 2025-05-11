//! Flash-card generation via LLM prompt - stub implementation.

use anyhow::Result;
use serde::Deserialize;

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
        // Build request body based on OpenAI Responses API.
        let instructions = "You are an expert pedagogue. For the given INPUT_TEXT produce concise flashcard JSON with keys front, back, tags (array of strings). Only output strict JSON.";

        let body = json!({
            "model": "gpt-4o-mini", // cheaper default; change as desired
            "instructions": instructions,
            "input": text,
            "temperature": 0.4,
            "max_output_tokens": 256,
            "text": { "format": { "type": "json_object" } }
        });

        // Get API key
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) => {
                tracing::info!("Found OpenAI API key (length: {})", key.len());
                key
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Environment variable OPENAI_API_KEY not set"));
            }
        };
        
        tracing::info!("Making OpenAI API request with model: {}", body["model"]);
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
            tracing::error!("OpenAI API error: Status={}, Body={}", status, error_body);
            return Err(anyhow::anyhow!("OpenAI API error: Status={}, Body={}", status, error_body));
        }

        tracing::info!("OpenAI API request successful");
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