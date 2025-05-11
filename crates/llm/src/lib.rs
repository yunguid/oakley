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

/// Generate card JSON from plain text.
pub async fn gen_card(text: &str) -> Result<CardFields> {
    #[cfg(feature = "full")]
    {
        let prompt = format!(
            "You are a pedagogue. Given INPUT_TEXT, output JSON with fields {{front, back, tags}}.\nINPUT_TEXT:\n{text}"
        );
        let raw = tokio::task::spawn_blocking(move || {
            llama_cpp::generate(&prompt, llama_cpp::Params::default())
        })
        .await??;
        
        Ok(serde_json::from_str(&raw)?)
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