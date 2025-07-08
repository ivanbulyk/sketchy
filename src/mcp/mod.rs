// src/mcp/mod.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageGenerationProvider {
    OpenAI,
    StabilityAI,
}

impl Default for ImageGenerationProvider {
    fn default() -> Self {
        ImageGenerationProvider::OpenAI
    }
}