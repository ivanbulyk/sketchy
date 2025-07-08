// src/services/redis_service.rs
use crate::errors::SketchyError;
use crate::models::*;
use redis::{AsyncCommands, Client};
use serde_json;
use uuid::Uuid;

pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub async fn new(redis_url: &str) -> Result<Self, SketchyError> {
        let client = Client::open(redis_url).map_err(|e| SketchyError::Redis(e.to_string()))?;

        // Test connection
        let mut conn = client
            .get_async_connection()
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        Ok(Self { client })
    }

    pub async fn store_image(&self, image: &ImageUpload) -> Result<(), SketchyError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        let key = format!("image:{}", image.id);
        let value =
            serde_json::to_string(image).map_err(|e| SketchyError::Serialization(e.to_string()))?;

        // Store with 24 hour expiration
        conn.set_ex::<_, _, ()>(&key, value, 86400)
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        // Add to session index
        let session_key = format!("session:{}:images", image.session_id);
        conn.sadd::<_, _, ()>(&session_key, image.id.to_string())
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        Ok(())
    }

    pub async fn get_image(&self, image_id: &Uuid) -> Result<ImageUpload, SketchyError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        let key = format!("image:{}", image_id);
        let value: String = conn
            .get(&key)
            .await
            .map_err(|e| SketchyError::Redis(format!("Image not found: {}", e)))?;

        serde_json::from_str(&value).map_err(|e| SketchyError::Serialization(e.to_string()))
    }

    pub async fn store_analysis(&self, analysis: &ImageAnalysis) -> Result<(), SketchyError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        let key = format!("analysis:{}", analysis.id);
        let value = serde_json::to_string(analysis)
            .map_err(|e| SketchyError::Serialization(e.to_string()))?;

        conn.set_ex::<_, _, ()>(&key, value, 86400)
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        // Index by image
        let image_key = format!("image:{}:analyses", analysis.image_id);
        conn.sadd::<_, _, ()>(&image_key, analysis.id.to_string())
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        Ok(())
    }

    pub async fn get_analysis(&self, analysis_id: &Uuid) -> Result<ImageAnalysis, SketchyError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        let key = format!("analysis:{}", analysis_id);
        let value: String = conn
            .get(&key)
            .await
            .map_err(|e| SketchyError::Redis(format!("Analysis not found: {}", e)))?;

        serde_json::from_str(&value).map_err(|e| SketchyError::Serialization(e.to_string()))
    }

    pub async fn store_regenerated(&self, image: &RegeneratedImage) -> Result<(), SketchyError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        let key = format!("regenerated:{}", image.id);
        let value =
            serde_json::to_string(image).map_err(|e| SketchyError::Serialization(e.to_string()))?;

        conn.set_ex::<_, _, ()>(&key, value, 86400)
            .await
            .map_err(|e| SketchyError::Redis(e.to_string()))?;

        Ok(())
    }
}
