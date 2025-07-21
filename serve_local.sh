#!/bin/bash
# 本地预览静态网站脚本

echo "🌐 Starting local preview server..."
echo "📁 Serving from: $(pwd)/public"
echo "🔗 Access at: http://localhost:9000"
echo ""
echo "Press Ctrl+C to stop the server"

cd public && python3 -m http.server 9000
