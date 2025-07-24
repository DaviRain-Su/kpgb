#!/bin/bash

echo "🧹 最终清理 - 只保留技术文章..."

# 设置为未发布的文章（功能演示、测试、个人笔记）
sqlite3 kpgb.db <<EOF
UPDATE posts SET published = 0 WHERE 
    title LIKE '%演示%' OR 
    title LIKE '%测试%' OR 
    title LIKE '%Test%' OR 
    title LIKE '%Demo%' OR
    title = '静态网站生成器完成' OR
    title = '禅者的初心📒' OR
    title = 'TingHu语录📒' OR
    title = '星荣英语01';
EOF

echo "✅ 已隐藏所有测试和个人笔记"

# 显示当前发布的文章
echo ""
echo "📝 当前发布的文章："
sqlite3 -column kpgb.db "SELECT title FROM posts WHERE published = 1 ORDER BY created_at DESC;" | head -20

# 重新生成网站
echo ""
echo "🔄 重新生成静态网站..."
cargo run generate

echo ""
echo "✅ 清理完成！您的博客现在只显示技术文章。"