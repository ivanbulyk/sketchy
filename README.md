# Sketchy - AI-Powered Image Learning Service

## Overview
Sketchy is a Rust-based web service designed to help users learn prompt engineering for AI image generation. It analyzes uploaded images, generates detailed descriptions, and then uses those descriptions to regenerate new images using various AI providers.

This project includes both a backend API and a frontend user interface.

## Features
- **Web UI:** A user-friendly interface to interact with all of the backend features.
- **Multi-Image Upload:** Upload one or more images to start a session.
- **High-Performance Storage:** Uses Redis for fast, temporary storage of image data and analysis results.
- **Multi-Provider Analysis:** Choose between different Large Language Models for image analysis:
    - OpenAI (`gpt-4o`)
    - Anthropic (`claude-3-5-sonnet-20241022`)
- **Multi-Provider Image Generation:** Generate images from prompts using:
    - OpenAI (`dall-e-3`)
    - Stability AI (`stable-image-ultra`)
- **Detailed Prompt Generation:** Creates a comprehensive prompt that can be used to recreate a similar image.
- **Flexible Regeneration:** Regenerate an image using the auto-generated prompt or provide your own custom prompt.
- **Chained Image Improvement:** Iteratively modify and improve generated images.

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

### Running the Application

1.  **Start your Redis server** if it's not already running:
    ```bash
    redis-server
    ```

2.  **Run the service:**
    ```bash
    cargo run
    ```

3.  **Open the Frontend:**
    The service will be available at [http://localhost:8080](http://localhost:8080). Open this URL in your web browser to use the application.

## API Endpoints

The frontend interacts with the following API endpoints.

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

### 5. Improve an Image (from Original)
Modify an original regenerated image based on a new prompt. This is the first step in an improvement chain.
- **Endpoint:** `POST /api/v1/improve/from_original/{regenerated_image_id}`
- **Body (JSON):**
    ```json
    {
        "prompt": "Change the season to winter, with snow on the ground."
    }
    ```
    - `prompt`: (Required) A new prompt to guide the image modification.
- **Returns:** A JSON object containing the `id` of the *newly improved* image and its base64-encoded `data`. This `id` can be used in the next endpoint for chained improvements.
    ```json
    {
        "id": "uuid-of-improved-image",
        "data": "base64-encoded-image-data"
    }
    ```

### 6. Improve an Image (Chained)
Perform a subsequent improvement on an *already improved* image.
- **Endpoint:** `POST /api/v1/improve/from_improved/{improved_image_id}`
- **Body (JSON):**
    ```json
    {
        "prompt": "Now add a snowman."
    }
    ```
    - `prompt`: (Required) A new prompt to guide the next image modification.
- **Returns:** A JSON object containing the `id` of the *next* improved image and its base64-encoded `data`, allowing for further chained calls.

## Architecture
- **Framework:** Actix-web
- **Frontend:** HTML, CSS, JavaScript
- **Storage:** Redis
- **AI Integrations:** OpenAI, Anthropic, Stability AI

## Development
```bash
# Run tests
cargo test

# Build for release
cargo build --release

# Run with detailed logging
RUST_LOG=info cargo run
```