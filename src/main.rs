// src/main.rs
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use log::info;
use std::sync::Arc;

mod errors;
mod handlers;
mod mcp;
mod models;
mod services;


use crate::handlers::{
    analyze_image, get_analysis, list_sessions, regenerate_image, upload_images,
};
use crate::services::{ImageProcessor, LLMService, RedisService};

#[derive(Clone)]
pub struct AppState {
    redis_service: Arc<RedisService>,
    llm_service: Arc<LLMService>,
    image_processor: Arc<ImageProcessor>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting Sketchy service...");

    // Initialize services
    let redis_service = Arc::new(RedisService::new("redis://127.0.0.1:6379").await.unwrap());
    let llm_service = Arc::new(LLMService::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"),
        std::env::var("ANTHROPIC_API_KEY").ok(),
        std::env::var("STABILITY_API_KEY").ok(),
    ));
    let image_processor = Arc::new(ImageProcessor::new());

    let app_state = AppState {
        redis_service,
        llm_service,
        image_processor,
    };

    info!("Starting HTTP server on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/upload", web::post().to(upload_images))
                    .route("/analyze/{image_id}", web::post().to(analyze_image))
                    .route("/analysis/{analysis_id}", web::get().to(get_analysis))
                    .route(
                        "/regenerate/{analysis_id}",
                        web::post().to(regenerate_image),
                    )
                    .route("/sessions", web::get().to(list_sessions)),
            )
            .route("/health", web::get().to(health_check))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "sketchy",
        "version": "0.1.0"
    }))
}
