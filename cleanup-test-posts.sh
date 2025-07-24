#!/bin/bash

# 清理所有测试相关的文章

echo "🧹 清理测试文章..."

# 要删除的文章ID前缀列表
TEST_POST_IDS=(
    "QmVY3Aqhj94j"  # Pagination Test Post 3
    "QmfMbK2SSPch"  # Pagination Test Post 2
    "QmWU4cL9EKYx"  # Pagination Test Post 1
    "QmeJyWFN1hw5"  # Tag System Demo Post (旧版)
    "Qmf29xVBTuy2"  # Auto Excerpt Test
    "QmNxdQkHHpTf"  # IPFS Test 1753067867
    "QmQmWyC1JXi2"  # Real IPFS Storage Test
    "49508d013251"  # My First IPFS Post
)

# 统计删除的文章数
deleted=0
failed=0

for id in "${TEST_POST_IDS[@]}"; do
    echo -n "删除 $id ... "
    if cargo run -- delete "$id" --force 2>&1 | grep -q "deleted successfully"; then
        echo "✅"
        ((deleted++))
    else
        echo "❌ (可能已删除或不存在)"
        ((failed++))
    fi
done

echo ""
echo "📊 清理结果："
echo "   成功删除: $deleted 篇"
echo "   跳过/失败: $failed 篇"
echo ""

# 重新生成网站
echo "🔄 重新生成静态网站..."
cargo run generate

echo ""
echo "✅ 清理完成！"