#!/bin/bash

echo "🧪 测试KPGB API端点"
echo "=================="

BASE_URL="http://localhost:3000"

echo ""
echo "1. 获取所有文章列表:"
curl -s "$BASE_URL/api/posts" | jq .

echo ""
echo "2. 搜索文章 (关键词: IPFS):"
curl -s -X POST "$BASE_URL/api/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "IPFS"}' | jq .

echo ""
echo "3. 获取单篇文章:"
# 获取第一篇文章的ID
FIRST_ID=$(curl -s "$BASE_URL/api/posts" | jq -r '.data[0].storage_id // empty')
if [ -n "$FIRST_ID" ]; then
    echo "   文章ID: $FIRST_ID"
    curl -s "$BASE_URL/api/posts/$FIRST_ID" | jq .
else
    echo "   没有找到文章"
fi

echo ""
echo "✅ API测试完成"