# Sketchy - AI-Powered Image Learning Service

## Overview
Sketchy is a Rust-based web service designed to help users learn prompt engineering for AI image generation. It analyzes uploaded images and generates detailed prompts that can recreate similar images.

## Features
- Multi-image upload (1-50 images)
- High-performance Redis storage
- LLM-based image analysis (OpenAI GPT-4 Vision, Anthropic Claude)
- Detailed prompt generation
- Image regeneration from prompts
- MCP tool integration

## Setup

### Prerequisites
- Rust 1.70+
- Redis server
- OpenAI API key
- Optional: Anthropic API key

### Environment Variables
```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"  # Optional
```

### Running
```bash
# Start Redis
redis-server

# Run the service
cargo run

# Service will be available at http://localhost:8080
```

## API Endpoints

### Upload Images
POST /api/v1/upload
- Multipart form data
- Returns session_id and image IDs

### Analyze Image
POST /api/v1/analyze/{image_id}?provider=openai
- Analyzes uploaded image
- Returns detailed analysis and generation prompt

### Get Analysis
GET /api/v1/analysis/{analysis_id}
- Retrieves stored analysis

### Regenerate Image
POST /api/v1/regenerate/{analysis_id}
- Optional custom prompt in body
- Returns generated image

## Architecture
- Actix-web for HTTP server
- Redis for in-memory storage
- Integration with OpenAI and Anthropic APIs
- MCP protocol support for tool extensions

## Development
```bash
# Run tests
cargo test

# Build release
cargo build --release

# Run with logging
RUST_LOG=info cargo run
```