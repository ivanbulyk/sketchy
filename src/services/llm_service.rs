// src/services/llm_service.rs
use crate::errors::SketchyError;
use crate::models::*;
use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

pub struct LLMService {
    openai_key: String,
    anthropic_key: Option<String>,
    client: Client,
}

impl LLMService {
    pub fn new(openai_key: String, anthropic_key: Option<String>) -> Self {
        Self {
            openai_key,
            anthropic_key,
            client: Client::new(),
        }
    }

    pub async fn analyze_image(
        &self,
        image_data: &[u8],
        provider: &str,
    ) -> Result<ImageAnalysis, SketchyError> {
        let start = Instant::now();

        match provider {
            "openai" => self.analyze_with_openai(image_data, start).await,
            "anthropic" => self.analyze_with_anthropic(image_data, start).await,
            _ => Err(SketchyError::InvalidProvider(provider.to_string())),
        }
    }

    async fn analyze_with_openai(
        &self,
        image_data: &[u8],
        start: Instant,
    ) -> Result<ImageAnalysis, SketchyError> {
        let base64_image = general_purpose::STANDARD.encode(image_data);

        let analysis_prompt = r#"
        Analyze this image in extreme detail for AI image generation. Provide:

        1. REGIONS: Identify all distinct regions/objects with:
           - Exact bounding box coordinates (x, y, width, height as percentages)
           - Dominant colors (hex codes with percentages)
           - Object description (what it is, texture, material)
           - Importance score (0-1)

        2. GLOBAL ATTRIBUTES:
           - Art style (photorealistic, cartoon, painting style, etc.)
           - Mood/atmosphere
           - Lighting (direction, quality, color temperature)
           - Camera perspective/angle
           - Overall dominant colors

        3. COMPOSITION:
           - Layout type (rule of thirds, centered, etc.)
           - Focal points (x,y coordinates as percentages)
           - Visual balance
           - Depth layers (foreground, midground, background elements)

        4. FORENSIC FACIAL RECONSTRUCTION:
           - If applicable, provide detailed facial features and expressions
           - Include attributes like age
           - Skin tone, hair color, eye color, etc.
           - Any visible scars, tattoos, or distinguishing marks
           - Facial symmetry and proportions

        5. GENERATION PROMPT:
           Create a detailed prompt that would recreate this image as closely as possible.
           Include all visual elements, style, composition, colors, and atmospheric details.
           Be extremely specific and comprehensive.

        Return as JSON matching this structure:
        {
            "regions": [...],
            "global_attributes": {...},
            "composition": {...},
            "generation_prompt": "..."
        }
        "#;

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&json!({
                "model": "gpt-4o",
                "messages": [{
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": analysis_prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/jpeg;base64,{}", base64_image)
                            }
                        }
                    ]
                }],
                "max_tokens": 4096,
                "response_format": { "type": "json_object" }
            }))
            .send()
            .await
            .map_err(|e| SketchyError::LLM(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SketchyError::LLM(format!("OpenAI error: {}", error_text)));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SketchyError::LLM(format!("Failed to parse OpenAI response: {}", e)))?;

        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| SketchyError::LLM("No content in OpenAI response".to_string()))?;

        let analysis_data: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| SketchyError::LLM(format!("Failed to parse analysis JSON: {}", e)))?;

        // Parse the analysis data into our structured format
        let raw_analysis = self.parse_raw_analysis(&analysis_data)?;
        let prompt_description = analysis_data["generation_prompt"]
            .as_str()
            .unwrap_or("Failed to generate prompt")
            .to_string();

        Ok(ImageAnalysis {
            id: Uuid::new_v4(),
            image_id: Uuid::new_v4(), // Will be set by handler
            llm_provider: "openai".to_string(),
            raw_analysis,
            prompt_description,
            metadata: AnalysisMetadata {
                processing_time_ms: start.elapsed().as_millis() as u64,
                model_used: "gpt-4o".to_string(),
                confidence_score: 0.85, // Could be calculated based on response
            },
            created_at: chrono::Utc::now(),
        })
    }

    async fn analyze_with_anthropic(
        &self,
        image_data: &[u8],
        start: Instant,
    ) -> Result<ImageAnalysis, SketchyError> {
        let api_key = self
            .anthropic_key
            .as_ref()
            .ok_or_else(|| SketchyError::LLM("Anthropic API key not configured".to_string()))?;

        let base64_image = general_purpose::STANDARD.encode(image_data);

        let analysis_prompt = r#"
        Analyze this image in extreme detail for AI image generation. Provide:

        1. REGIONS: Identify all distinct regions/objects with:
           - Exact bounding box coordinates (x, y, width, height as percentages)
           - Dominant colors (hex codes with percentages)
           - Object description (what it is, texture, material)
           - Importance score (0-1)

        2. GLOBAL ATTRIBUTES:
           - Art style (photorealistic, cartoon, painting style, etc.)
           - Mood/atmosphere
           - Lighting (direction, quality, color temperature)
           - Camera perspective/angle
           - Overall dominant colors

        3. COMPOSITION:
           - Layout type (rule of thirds, centered, etc.)
           - Focal points (x,y coordinates as percentages)
           - Visual balance
           - Depth layers (foreground, midground, background elements)

        4. FORENSIC FACIAL RECONSTRUCTION:
           - If applicable, provide detailed facial features and expressions
           - Include attributes like age
           - Skin tone, hair color, eye color, etc.
           - Any visible scars, tattoos, or distinguishing marks
           - Facial symmetry and proportions

        5. GENERATION PROMPT:
           Create a detailed prompt that would recreate this image as closely as possible.
           Include all visual elements, style, composition, colors, and atmospheric details.
           Be extremely specific and comprehensive.

        Return as JSON matching this structure:
        {
            "regions": [...],
            "global_attributes": {...},
            "composition": {...},
            "generation_prompt": "..."
        }
        "#;

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "claude-3-5-sonnet-20241022",
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": analysis_prompt
                        },
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/jpeg",
                                "data": base64_image
                            }
                        }
                    ]
                }]
            }))
            .send()
            .await
            .map_err(|e| SketchyError::LLM(format!("Anthropic request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SketchyError::LLM(format!(
                "Anthropic error: {}",
                error_text
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SketchyError::LLM(format!("Failed to parse Anthropic response: {}", e)))?;

        let content = result["content"][0]["text"]
            .as_str()
            .ok_or_else(|| SketchyError::LLM("No content in Anthropic response".to_string()))?;

        let analysis_data: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| SketchyError::LLM(format!("Failed to parse analysis JSON: {}", e)))?;

        // Parse the analysis data into our structured format
        let raw_analysis = self.parse_raw_analysis(&analysis_data)?;
        let prompt_description = analysis_data["generation_prompt"]
            .as_str()
            .unwrap_or("Failed to generate prompt")
            .to_string();

        Ok(ImageAnalysis {
            id: Uuid::new_v4(),
            image_id: Uuid::new_v4(), // Will be set by handler
            llm_provider: "anthropic".to_string(),
            raw_analysis,
            prompt_description,
            metadata: AnalysisMetadata {
                processing_time_ms: start.elapsed().as_millis() as u64,
                model_used: "claude-3-5-sonnet-20241022".to_string(),
                confidence_score: 0.85, // Could be calculated based on response
            },
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn generate_image(
        &self,
        prompt: &str,
        format: &str,
    ) -> Result<RegeneratedImage, SketchyError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/images/generations")
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&json!({
                "model": "dall-e-3",
                "prompt": prompt,
                "n": 1,
                "size": "1024x1024",
                "quality": "hd",
                "response_format": "b64_json"
            }))
            .send()
            .await
            .map_err(|e| SketchyError::LLM(format!("Image generation request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SketchyError::LLM(format!(
                "Image generation error: {}",
                error_text
            )));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| {
            SketchyError::LLM(format!("Failed to parse generation response: {}", e))
        })?;

        let b64_json = result["data"][0]["b64_json"]
            .as_str()
            .ok_or_else(|| SketchyError::LLM("No image data in response".to_string()))?;

        let image_data = general_purpose::STANDARD
            .decode(b64_json)
            .map_err(|e| SketchyError::LLM(format!("Failed to decode image: {}", e)))?;

        Ok(RegeneratedImage {
            id: Uuid::new_v4(),
            analysis_id: Uuid::new_v4(), // Will be set by handler
            format: ImageFormat::Raster {
                format: "png".to_string(),
                dimensions: (1024, 1024),
            },
            data: image_data,
            prompt_used: prompt.to_string(),
            generation_params: GenerationParams {
                model: "dall-e-3".to_string(),
                steps: None,
                cfg_scale: None,
                seed: None,
            },
            created_at: chrono::Utc::now(),
        })
    }

    fn parse_raw_analysis(&self, data: &serde_json::Value) -> Result<RawAnalysis, SketchyError> {
        // Parse regions
        let regions = data["regions"]
            .as_array()
            .ok_or_else(|| SketchyError::LLM("Missing regions in analysis".to_string()))?
            .iter()
            .map(|r| {
                Ok(ImageRegion {
                    id: Uuid::new_v4().to_string(),
                    coordinates: BoundingBox {
                        x: r["coordinates"]["x"].as_u64().unwrap_or(0) as u32,
                        y: r["coordinates"]["y"].as_u64().unwrap_or(0) as u32,
                        width: r["coordinates"]["width"].as_u64().unwrap_or(0) as u32,
                        height: r["coordinates"]["height"].as_u64().unwrap_or(0) as u32,
                    },
                    dominant_colors: self.parse_colors(r["dominant_colors"].as_array()),
                    object_description: r["object_description"].as_str().unwrap_or("").to_string(),
                    texture_description: r["texture_description"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    importance_score: r["importance_score"].as_f64().unwrap_or(0.5) as f32,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Parse global attributes
        let global = &data["global_attributes"];
        let global_attributes = GlobalAttributes {
            style: global["style"].as_str().unwrap_or("unknown").to_string(),
            mood: global["mood"].as_str().unwrap_or("neutral").to_string(),
            lighting: global["lighting"].as_str().unwrap_or("natural").to_string(),
            perspective: global["perspective"]
                .as_str()
                .unwrap_or("eye-level")
                .to_string(),
            dominant_colors: self.parse_colors(global["dominant_colors"].as_array()),
        };

        // Parse composition
        let comp = &data["composition"];
        let composition = CompositionAnalysis {
            layout: comp["layout"].as_str().unwrap_or("centered").to_string(),
            focal_points: comp["focal_points"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            let x = p["x"].as_f64()? as f32;
                            let y = p["y"].as_f64()? as f32;
                            Some((x, y))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            balance: comp["balance"].as_str().unwrap_or("symmetric").to_string(),
            depth_layers: comp["depth_layers"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        };

        Ok(RawAnalysis {
            regions,
            global_attributes,
            composition,
        })
    }

    fn parse_colors(&self, colors_array: Option<&Vec<serde_json::Value>>) -> Vec<Color> {
        colors_array
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let hex = c["hex"].as_str()?.to_string();
                        let rgb = (
                            c["rgb"][0].as_u64()? as u8,
                            c["rgb"][1].as_u64()? as u8,
                            c["rgb"][2].as_u64()? as u8,
                        );
                        let percentage = c["percentage"].as_f64()? as f32;
                        Some(Color {
                            hex,
                            rgb,
                            percentage,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
