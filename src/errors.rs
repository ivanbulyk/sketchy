// src/errors.rs
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SketchyError {
    #[error("Redis error: {0}")]
    Redis(String),

    #[error("LLM service error: {0}")]
    LLM(String),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid provider: {0}")]
    InvalidProvider(String),
}

impl ResponseError for SketchyError {
    fn error_response(&self) -> HttpResponse {
        match self {
            SketchyError::Redis(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": self.to_string()
            })),
            SketchyError::LLM(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "AI service error",
                "message": self.to_string()
            })),
            SketchyError::ImageProcessing(_) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Image processing error",
                    "message": self.to_string()
                }))
            }
            SketchyError::Serialization(_) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Data processing error",
                    "message": self.to_string()
                }))
            }
            SketchyError::Validation(_) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Validation error",
                "message": self.to_string()
            })),
            SketchyError::InvalidProvider(_) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid provider",
                    "message": self.to_string()
                }))
            }
        }
    }
}
