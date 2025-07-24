#!/bin/bash

# 批量删除测试文章的脚本

echo "🗑️  删除测试文章..."
echo "⚠️  警告：此操作不可撤销！"
echo ""

# 要删除的测试文章ID列表
TEST_POSTS=(
    # 分页测试文章
    "QmVY3Aqhj94jdJLj4ituS8yNZ6WSx4hoA4NRpmivGeVx6w"  # Pagination Test Post 3
    "QmfMbK2SSPchbyhHz5pPYbdJZXgZJrkUpsE4ysEymDbruo"  # Pagination Test Post 2
    "QmWU4cL9EKYxzt5ds8uWsFS2btzRZ8GtE5SN5Q9GauktJe"  # Pagination Test Post 1
    
    # 标签系统演示（重复的）
    "QmeJyWFN1hw53QNc581BT9SQ4dZVPZQm1WSyGTC1FPd5dk"  # Tag System Demo Post (旧版)
    
    # 其他测试文章
    "Qmf29xVBTuy2eUFHHtZQecxBdQDNxLEkWZeQULFS9QArst"  # Auto Excerpt Test
    "QmNxdQkHHpTfXW9MsHJ9kmZMHXukhBKUPeatNijy2iAr6n"  # IPFS Test 1753067867
    "49508d01325161cea625d5dfa585c4166c1736686b07e1c575769e65e1831539"  # My First IPFS Post
)

echo "将要删除以下文章："
for id in "${TEST_POSTS[@]}"; do
    echo "  - $id"
done

echo ""
read -p "确认删除这些文章吗？(y/N) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    for id in "${TEST_POSTS[@]}"; do
        echo "删除: $id"
        cargo run -- delete "$id" --force
    done
    echo "✅ 删除完成"
else
    echo "❌ 取消删除"
fi