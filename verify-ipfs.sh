#!/bin/bash

echo "🔍 验证IPFS存储"
echo "=============="

# 检查IPFS是否安装
if ! command -v ipfs &> /dev/null; then
    echo "❌ IPFS未安装"
    echo "请先安装IPFS: brew install ipfs"
    exit 1
fi

echo "✅ IPFS已安装: $(ipfs version)"

# 检查IPFS daemon是否运行
if ! curl -s -X POST http://localhost:5001/api/v0/version > /dev/null 2>&1; then
    echo "❌ IPFS daemon未运行"
    echo ""
    echo "请在新终端运行: ipfs daemon"
    echo "然后重新运行此脚本"
    exit 1
fi

echo "✅ IPFS daemon正在运行"

# 如果IPFS正在运行，创建一个测试文章
echo ""
echo "📝 创建测试文章..."

# 创建测试内容
cat > ipfs-test.md << EOF
# IPFS存储测试

这是一个存储在IPFS上的测试文章。

时间戳: $(date)
EOF

# 使用IPFS后端创建文章
echo ""
echo "正在使用IPFS后端创建文章..."
IPFS_API_URL=http://localhost:5001 cargo run -q -- new --title "IPFS Test $(date +%s)" --author "IPFS Tester" --content ipfs-test.md

# 获取最新的文章ID
echo ""
echo "📋 最新创建的文章："
IPFS_API_URL=http://localhost:5001 cargo run -q -- list | grep -A 5 "ID:" | head -6

# 清理
rm ipfs-test.md

echo ""
echo "💡 如何验证内容在IPFS上："
echo "1. 查看上面的Storage ID (应该是Qm开头的CID)"
echo "2. 使用命令: ipfs cat <CID>"
echo "3. 或访问: http://localhost:8080/ipfs/<CID>"
echo "4. 或使用公共网关: https://ipfs.io/ipfs/<CID>"