#!/bin/bash

# 批量取消发布某些文章的脚本

echo "🔒 取消发布测试和不需要的文章..."

# 测试相关文章
POSTS_TO_UNPUBLISH=(
    # 功能演示文章（根据需要取消注释）
    # "QmdgptYDf5Bo48w1mBsY6EFEyuu1tALJNUeHhdRaRM6D9p"  # 脚注悬浮显示功能演示
    # "QmQAdgVSLMH19dVwi4dLhdkmT5SEgWVcUbM5SbU9XbTigv"  # 测试图片灯箱效果
    # "QmWjDyAFCDBUhcVJmviTF7V7ndWYFNPbVXzzdoiTgtpQ9e"  # IPFS图片自动上传测试
    
    # 分页测试文章
    "QmVY3Aqhj94jdJLj4ituS8yNZ6WSx4hoA4NRpmivGeVx6w"  # Pagination Test Post 3
    "QmfMbK2SSPchbyhHz5pPYbdJZXgZJrkUpsE4ysEymDbruo"  # Pagination Test Post 2
    "QmWU4cL9EKYxzt5ds8uWsFS2btzRZ8GtE5SN5Q9GauktJe"  # Pagination Test Post 1
    
    # 标签系统演示
    "QmfTnU8jYJTL2da8YDQjSX9iJYE3LEQNtnRzDAwJaTPtWC"  # Tag System Demo Post
    "QmeJyWFN1hw53QNc581BT9SQ4dZVPZQm1WSyGTC1FPd5dk"  # Tag System Demo Post (duplicate)
    
    # 其他测试文章
    "Qmf29xVBTuy2eUFHHtZQecxBdQDNxLEkWZeQULFS9QArst"  # Auto Excerpt Test
    "QmQmWyC1JXi269pT6J6Jnip9mP5aWHcquDfd7DFCjAYFo2"  # Real IPFS Storage Test
    
    # 个人笔记（根据需要取消注释）
    # "QmPEiHRYWUL85YYWPKVXCscJNL1Ka1u9hEv3BUFZLiXw1i"  # 禅者的初心📒
    # "QmYBGJGdy6aGmX6D5VvXhFFdsVdbkKzgsyVo3LnxGtUcut"  # TingHu语录📒
    # "QmU243MrDk9VUwMcNSW7QPfPQRDTeJSdj6SYGcW5mEh7kY"  # 星荣英语01
)

# 使用数据库直接更新（更快）
echo "UPDATE posts SET published = 0 WHERE storage_id IN ($(printf "'%s'," "${POSTS_TO_UNPUBLISH[@]}" | sed 's/,$//'))" | sqlite3 kpgb.db

echo "✅ 已取消发布 ${#POSTS_TO_UNPUBLISH[@]} 篇文章"
echo ""
echo "📝 如需重新发布，使用："
echo "   cargo run -- publish <文章ID>"