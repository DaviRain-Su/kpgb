#!/bin/bash

echo "🌐 启动本地HTTP服务器..."
echo "访问: http://localhost:8000"
echo "按 Ctrl+C 停止服务器"
echo ""

cd public && python3 -m http.server 8000