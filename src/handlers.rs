// src/handlers.rs
use crate::{AppState, errors::SketchyError, models::*};
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, web};
use futures_util::TryStreamExt;
use uuid::Uuid;

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
    body: web::Json<serde_json::Value>,
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
        .get("prompt")
        .and_then(|p| p.as_str())
        .unwrap_or(&analysis.prompt_description);

    let format = body
        .get("format")
        .and_then(|f| f.as_str())
        .unwrap_or("raster");

    // Generate image
    let mut regenerated = data
        .llm_service
        .generate_image(prompt, format)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    regenerated.analysis_id = analysis_id;

    // Store regenerated image
    data.redis_service
        .store_regenerated(&regenerated)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Return image data
    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(regenerated.data))
}

pub async fn list_sessions(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    // This would implement listing recent sessions
    // For now, return a placeholder
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "sessions": [],
        "message": "Session listing not yet implemented"
    })))
}
