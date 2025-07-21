#!/bin/bash
# Preview script for static site with theme

CONFIG_FILE="${1:-site.toml}"

echo "📋 Using config: $CONFIG_FILE"
echo "🎨 Current theme: $(grep '^theme' "$CONFIG_FILE" | cut -d'"' -f2)"
echo "🚀 Generating static site..."
cargo run -- generate --config "$CONFIG_FILE"

echo "🌐 Starting local preview server..."
echo "📍 Visit: http://localhost:8888"
echo "❌ Press Ctrl+C to stop"

cd public && python3 -m http.server 8888