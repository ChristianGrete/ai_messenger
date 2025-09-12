#!/bin/bash
set -euo pipefail

# Build script for Ollama LLM adapter (testing purposes only)
# This script builds the WASM component and deploys it to the data directory
# for development and testing. In production, adapters will be separate repositories.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADAPTER_DIR="$PROJECT_ROOT/adapters/llm/ollama"
ADAPTER_VERSION="0.0.1-alpha"

echo "🔧 Building Ollama LLM adapter..."

# Get data directory from the main binary
DATA_DIR="$(cd "$PROJECT_ROOT" && cargo run -- data)"
TARGET_DIR="$DATA_DIR/adapters/llm/ollama/$ADAPTER_VERSION"
LATEST_DIR="$DATA_DIR/adapters/llm/ollama/latest"

echo "📍 Data directory: $DATA_DIR"
echo "🎯 Target directory: $TARGET_DIR"

# Build the WASM component
echo "🏗️  Building WASM component..."
cd "$ADAPTER_DIR"
cargo component build --release

# Create target directories
mkdir -p "$TARGET_DIR"
mkdir -p "$LATEST_DIR"

# Copy the built component
echo "📦 Copying adapter.wasm..."
cp "$ADAPTER_DIR/target/wasm32-wasip1/release/ollama_adapter.wasm" "$TARGET_DIR/adapter.wasm"

# Copy manifest
echo "📋 Copying manifest.json..."
cp "$ADAPTER_DIR/manifest.json" "$TARGET_DIR/manifest.json"

# Update symlinks for latest
echo "🔗 Updating latest symlinks..."
cd "$DATA_DIR/adapters/llm/ollama"
rm -f latest/adapter.wasm latest/manifest.json
ln -sf "../$ADAPTER_VERSION/adapter.wasm" latest/adapter.wasm
ln -sf "../$ADAPTER_VERSION/manifest.json" latest/manifest.json

echo "✅ Ollama adapter build complete!"
echo "📄 Component: $TARGET_DIR/adapter.wasm"
echo "📄 Manifest: $TARGET_DIR/manifest.json"
echo "🔗 Latest: $LATEST_DIR/ (symlinked)"

# Verify the component (check the actual file, not symlink)
echo "🔍 Verifying component format..."
file "$TARGET_DIR/adapter.wasm"

echo ""
echo "🚀 Ready to test! Start the server and make requests."
