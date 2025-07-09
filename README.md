# Sketchy - AI-Powered Image Learning Service

## Overview
Sketchy is a Rust-based web service designed to help users learn prompt engineering for AI image generation. It analyzes uploaded images, generates detailed descriptions, and then uses those descriptions to regenerate new images using various AI providers.

## Features
- **Multi-Image Upload:** Upload up to 50 images in a single session.
- **High-Performance Storage:** Uses Redis for fast, temporary storage of image data and analysis results.
- **Multi-Provider Analysis:** Choose between different Large Language Models for image analysis:
    - OpenAI (`gpt-4o`)
    - Anthropic (`claude-3-5-sonnet-20241022`)
- **Multi-Provider Image Generation:** Generate images from prompts using:
    - OpenAI (`dall-e-3`)
    - Stability AI (`stable-image-ultra`)
- **Detailed Prompt Generation:** Creates a comprehensive prompt that can be used to recreate a similar image.
- **Flexible Regeneration:** Regenerate an image using the auto-generated prompt or provide your own custom prompt.

## Setup

### Prerequisites
- Rust 1.70+
- Redis server
- API Keys for your chosen providers.

### Environment Variables
Create a `.env` file or export the following environment variables:
```bash
# Required
export OPENAI_API_KEY="your-openai-key"

# Optional - Add keys for the services you want to use
export ANTHROPIC_API_KEY="your-anthropic-key"
export STABILITY_API_KEY="your-stability-ai-key"
```

### Running
```bash
# Start your Redis server if it's not already running
redis-server

# Run the service
cargo run

# The service will be available at http://localhost:8080
```

## API Endpoints

### 1. Upload Images
Upload one or more images to start a session.
- **Endpoint:** `POST /api/v1/upload`
- **Body:** Multipart form data with images.
- **Returns:** A `session_id` and a list of `image_id`s for the uploaded files.

### 2. Analyze an Image
Submit an image for analysis by an LLM provider.
- **Endpoint:** `POST /api/v1/analyze/{image_id}`
- **Query Parameter:** `?provider={openai|anthropic}` (defaults to `openai`)
- **Returns:** A detailed analysis, including a generated `prompt_description` and an `analysis_id`.

### 3. Get Analysis Results
Retrieve the stored analysis for a given ID.
- **Endpoint:** `GET /api/v1/analysis/{analysis_id}`
- **Returns:** The full analysis JSON.

### 4. Regenerate an Image
Generate a new image based on the analysis.
- **Endpoint:** `POST /api/v1/regenerate/{analysis_id}`
- **Body (JSON):**
    ```json
    {
        "provider": "stabilityai",
        "prompt": "A custom prompt to override the generated one."
    }
    ```
    - `provider`: (Optional) `openai` or `stabilityai`. Defaults to `openai`.
    - `prompt`: (Optional) If omitted, the `prompt_description` from the analysis will be used.
    - `style_preset`: (Optional) A specific style preset to apply to the generated image (e.g., `photographic`, `anime`, `digital-art`). Only applicable for Stability AI.
- **Returns:** A JSON object containing the `id` of the regenerated image and its base64-encoded `data`.
    ```json
    {
        "id": "uuid-of-regenerated-image",
        "data": "base64-encoded-image-data"
    }
    ```

### 5. Improve an Image
Modify an existing regenerated image based on a new prompt.
- **Endpoint:** `POST /api/v1/improve/{regenerated_image_id}`
- **Body (JSON):**
    ```json
    {
        "prompt": "Change the season to winter, with snow on the ground."
    }
    ```
    - `prompt`: (Required) A new prompt to guide the image modification.
- **Returns:** The improved image file (e.g., a PNG).

## Architecture
- **Framework:** Actix-web
- **Storage:** Redis
- **AI Integrations:** OpenAI, Anthropic, Stability AI
- **Extensibility:** MCP protocol support for tool extensions

## Development
```bash
# Run tests
cargo test

# Build for release
cargo build --release

# Run with detailed logging
RUST_LOG=info cargo run
```
