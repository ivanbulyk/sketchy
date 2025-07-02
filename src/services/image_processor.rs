// src/services/image_processor.rs
use crate::errors::SketchyError;
use image::{DynamicImage, GenericImageView, ImageFormat as ImgFormat};

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_image(&self, data: &[u8]) -> Result<(u32, u32), SketchyError> {
        let img = image::load_from_memory(data)
            .map_err(|e| SketchyError::ImageProcessing(format!("Invalid image format: {}", e)))?;

        let (width, height) = img.dimensions();

        // Check image size limits
        if width > 4096 || height > 4096 {
            return Err(SketchyError::ImageProcessing(
                "Image dimensions exceed 4096x4096".to_string(),
            ));
        }

        Ok((width, height))
    }

    pub fn resize_if_needed(&self, data: &[u8], max_size: u32) -> Result<Vec<u8>, SketchyError> {
        let img = image::load_from_memory(data)
            .map_err(|e| SketchyError::ImageProcessing(format!("Failed to load image: {}", e)))?;

        let (width, height) = img.dimensions();

        if width <= max_size && height <= max_size {
            return Ok(data.to_vec());
        }

        let ratio = (max_size as f32 / width.max(height) as f32).min(1.0);
        let new_width = (width as f32 * ratio) as u32;
        let new_height = (height as f32 * ratio) as u32;

        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

        let mut output = Vec::new();
        resized
            .write_to(&mut std::io::Cursor::new(&mut output), ImgFormat::Png)
            .map_err(|e| {
                SketchyError::ImageProcessing(format!("Failed to encode resized image: {}", e))
            })?;

        Ok(output)
    }

    pub fn resize_for_anthropic(&self, data: &[u8]) -> Result<Vec<u8>, SketchyError> {
        // Anthropic has a 5MB limit for base64 encoded images
        // Base64 encoding increases size by ~33%, so we need to keep raw image under ~3.75MB
        const MAX_SIZE_BYTES: usize = 3_750_000; // ~3.75MB to account for base64 encoding
        
        if data.len() <= MAX_SIZE_BYTES {
            return Ok(data.to_vec());
        }

        let img = image::load_from_memory(data)
            .map_err(|e| SketchyError::ImageProcessing(format!("Failed to load image: {}", e)))?;

        let (width, height) = img.dimensions();
        
        // Calculate scale factor to reduce file size
        let scale_factor = ((MAX_SIZE_BYTES as f64 / data.len() as f64).sqrt() * 0.9) as f32;
        let new_width = ((width as f32 * scale_factor) as u32).max(256);
        let new_height = ((height as f32 * scale_factor) as u32).max(256);

        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

        let mut output = Vec::new();
        resized
            .write_to(&mut std::io::Cursor::new(&mut output), ImgFormat::Jpeg)
            .map_err(|e| {
                SketchyError::ImageProcessing(format!("Failed to encode resized image: {}", e))
            })?;

        Ok(output)
    }
}
