// src/handlers.rs
use crate::{AppState, errors::SketchyError, mcp::ImageGenerationProvider, models::*};
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, web};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

#[derive(Deserialize)]
pub struct RegenerateImageBody {
    prompt: Option<String>,
    provider: Option<ImageGenerationProvider>,
    format: Option<String>,
    style_preset: Option<String>,
}

#[derive(Serialize)]
pub struct RegenerateImageResponse {
    pub id: Uuid,
    pub data: String, // Base64 encoded image data
}

#[derive(Deserialize)]
pub struct ImproveImageBody {
    prompt: String,
    // Add other parameters for image improvement if needed, e.g., strength
}

#[derive(Serialize)]
pub struct ImproveImageResponse {
    pub id: Uuid,
    pub data: String, // Base64 encoded image data
}

pub async fn upload_images(
    mut payload: Multipart,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session_id = Uuid::new_v4();
    let mut uploaded_images = Vec::new();

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .ok_or_else(|| SketchyError::Validation("No filename provided".to_string()))?
            .to_string();

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Collect image data
        let mut image_data = Vec::new();
        while let Some(chunk) = field.try_next().await? {
            image_data.extend_from_slice(&chunk);
        }

        // Validate image
        data.image_processor
            .validate_image(&image_data)
            .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

        // Resize if needed
        let processed_data = data
            .image_processor
            .resize_if_needed(&image_data, 2048)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let image_upload = ImageUpload {
            id: Uuid::new_v4(),
            session_id,
            filename,
            content_type,
            size: processed_data.len(),
            data: processed_data,
            uploaded_at: chrono::Utc::now(),
        };

        // Store in Redis
        data.redis_service
            .store_image(&image_upload)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        uploaded_images.push(image_upload.id);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "session_id": session_id,
        "uploaded_images": uploaded_images,
        "count": uploaded_images.len()
    })))
}

pub async fn analyze_image(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let image_id = path.into_inner();
    let provider = query
        .get("provider")
        .map(|s| s.as_str())
        .unwrap_or("openai");

    // Retrieve image from Redis
    let image = data
        .redis_service
        .get_image(&image_id)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // Resize image for Anthropic if needed (5MB limit)
    let image_data = if provider == "anthropic" {
        data.image_processor
            .resize_for_anthropic(&image.data)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else {
        image.data.clone()
    };

    // Analyze with LLM
    let mut analysis = data
        .llm_service
        .analyze_image(&image_data, provider)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    analysis.image_id = image_id;

    // Store analysis
    data.redis_service
        .store_analysis(&analysis)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(&analysis))
}

pub async fn get_analysis(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let analysis_id = path.into_inner();

    let analysis = data
        .redis_service
        .get_analysis(&analysis_id)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    Ok(HttpResponse::Ok().json(&analysis))
}

pub async fn regenerate_image(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    body: web::Json<RegenerateImageBody>,
) -> Result<HttpResponse, Error> {
    let analysis_id = path.into_inner();

    // Get analysis
    let analysis = data
        .redis_service
        .get_analysis(&analysis_id)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // Use custom prompt if provided, otherwise use the generated one
    let prompt = body
        .prompt
        .as_deref()
        .unwrap_or(&analysis.prompt_description);

    let provider = body.provider.clone().unwrap_or_default();
    let format = body.format.as_deref().unwrap_or("raster");
    let style_preset = body.style_preset.as_deref();

    // Generate image
    let mut regenerated = data
        .llm_service
        .generate_image(prompt, provider, format, style_preset)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    regenerated.analysis_id = analysis_id;

    // Store regenerated image
    data.redis_service
        .store_regenerated(&regenerated)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Return image data
    Ok(HttpResponse::Ok().json(RegenerateImageResponse {
        id: regenerated.id,
        data: general_purpose::STANDARD.encode(&regenerated.data),
    }))
}

pub async fn improve_image(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    body: web::Json<ImproveImageBody>,
) -> Result<HttpResponse, Error> {
    let regenerated_image_id = path.into_inner();

    // Retrieve the original regenerated image from Redis
    let original_image = data
        .redis_service
        .get_regenerated(&regenerated_image_id)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // Use the custom prompt for improvement
    let prompt = body.prompt.as_str();

    // Call the LLM service to improve the image
    let mut improved_image = data
        .llm_service
        .improve_image(&original_image.data, prompt)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    improved_image.regenerated_image_id = regenerated_image_id;

    // Store the improved image
    data.redis_service
        .store_improved(&improved_image)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Return the improved image data
    Ok(HttpResponse::Ok().json(ImproveImageResponse {
        id: improved_image.id,
        data: general_purpose::STANDARD.encode(&improved_image.data),
    }))
}

pub async fn improve_from_improved(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    body: web::Json<ImproveImageBody>,
) -> Result<HttpResponse, Error> {
    let improved_image_id = path.into_inner();

    // Retrieve the previous improved image from Redis
    let previous_image = data
        .redis_service
        .get_improved(&improved_image_id)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // Use the custom prompt for improvement
    let prompt = body.prompt.as_str();

    // Call the LLM service to improve the image
    let mut new_improved_image = data
        .llm_service
        .improve_image(&previous_image.data, prompt)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // The new image still points back to the original regenerated image
    new_improved_image.regenerated_image_id = previous_image.regenerated_image_id;

    // Store the new improved image
    data.redis_service
        .store_improved(&new_improved_image)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Return the new improved image's ID and data
    Ok(HttpResponse::Ok().json(ImproveImageResponse {
        id: new_improved_image.id,
        data: general_purpose::STANDARD.encode(&new_improved_image.data),
    }))
}

pub async fn list_sessions(_data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    // This would implement listing recent sessions
    // For now, return a placeholder
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "sessions": [],
        "message": "Session listing not yet implemented"
    })))
}
