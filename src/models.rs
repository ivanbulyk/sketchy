// src/models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUpload {
    pub id: Uuid,
    pub session_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub data: Vec<u8>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysis {
    pub id: Uuid,
    pub image_id: Uuid,
    pub llm_provider: String,
    pub raw_analysis: RawAnalysis,
    pub prompt_description: String,
    pub metadata: AnalysisMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAnalysis {
    pub regions: Vec<ImageRegion>,
    pub global_attributes: GlobalAttributes,
    pub composition: CompositionAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRegion {
    pub id: String,
    pub coordinates: BoundingBox,
    pub dominant_colors: Vec<Color>,
    pub object_description: String,
    pub texture_description: String,
    pub importance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub hex: String,
    pub rgb: (u8, u8, u8),
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAttributes {
    pub style: String,
    pub mood: String,
    pub lighting: String,
    pub perspective: String,
    pub dominant_colors: Vec<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionAnalysis {
    pub layout: String,
    pub focal_points: Vec<(f32, f32)>,
    pub balance: String,
    pub depth_layers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub processing_time_ms: u64,
    pub model_used: String,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegeneratedImage {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub prompt_used: String,
    pub generation_params: GenerationParams,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Raster {
        format: String,
        dimensions: (u32, u32),
    },
    Vector {
        format: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub model: String,
    pub steps: Option<u32>,
    pub cfg_scale: Option<f32>,
    pub seed: Option<i64>,
}
