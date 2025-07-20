#!/bin/bash

echo "🔍 检查当前存储配置"
echo "==================="

# 检查环境变量
if [ -f .env ]; then
    echo "📄 .env文件内容:"
    cat .env | grep -E "(IPFS_API_URL|DATABASE_URL)"
    echo ""
fi

# 检查IPFS连接
echo "🌐 IPFS连接状态:"
if curl -s -X POST http://localhost:5001/api/v0/version > /dev/null 2>&1; then
    echo "✅ IPFS daemon正在运行 (http://localhost:5001)"
    echo "   默认存储后端将使用: IPFS"
else
    echo "❌ IPFS daemon未运行"
    echo "   默认存储后端将使用: Local"
fi

echo ""
echo "📁 本地存储内容:"
if [ -d storage/local ]; then
    count=$(ls -1 storage/local 2>/dev/null | wc -l)
    echo "   本地存储文件数: $count"
    if [ $count -gt 0 ]; then
        echo "   最新文件:"
        ls -lt storage/local | head -4
    fi
else
    echo "   本地存储目录不存在"
fi

echo ""
echo "🗄️ 数据库内容:"
if [ -f kpgb.db ]; then
    sqlite3 kpgb.db "SELECT COUNT(*) as count FROM posts;" 2>/dev/null | while read count; do
        echo "   文章总数: $count"
    done
    echo ""
    echo "   最新文章:"
    sqlite3 kpgb.db "SELECT storage_id, title, created_at FROM posts ORDER BY created_at DESC LIMIT 3;" 2>/dev/null | while IFS='|' read id title created; do
        # 检查是否是IPFS CID
        if [[ $id == Qm* ]]; then
            echo "   ✅ IPFS: $id - $title"
        else
            echo "   📁 Local: ${id:0:16}... - $title"
        fi
    done
else
    echo "   数据库文件不存在"
fi

echo ""
echo "💡 提示:"
echo "   - 要使用IPFS存储，请先运行: ipfs daemon"
echo "   - 当前如果IPFS未运行，内容将存储在本地"
echo "   - 使用 ./verify-ipfs.sh 来测试IPFS存储"