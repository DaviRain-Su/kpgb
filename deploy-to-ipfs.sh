#!/bin/bash

# Deploy KPGB static site to IPFS for .sol domain

echo "🚀 Deploying KPGB to IPFS..."

# Build the static site
echo "📦 Building static site..."
cargo run generate

# Add to IPFS
echo "📤 Uploading to IPFS..."
IPFS_HASH=$(ipfs add -r ./public --quieter)

echo "✅ Site deployed to IPFS!"
echo "📌 IPFS Hash: $IPFS_HASH"
echo "🌐 IPFS Gateway URL: https://ipfs.io/ipfs/$IPFS_HASH"
echo ""
echo "🔗 To set this as your .sol domain's IPFS record:"
echo "   1. Go to https://v1.sns.id"
echo "   2. Find davirainio.sol"
echo "   3. Set IPFS record to: $IPFS_HASH"
echo ""
echo "📱 Your domain will be accessible at:"
echo "   - https://davirainio.sol-domain.org (proxy)"
echo "   - davirainio.sol (in Brave browser)"

# Optional: Pin to public gateway
echo ""
echo "📌 Pinning to public gateways..."
curl -X POST "https://ipfs.io/api/v0/pin/add?arg=/ipfs/$IPFS_HASH" 2>/dev/null || true