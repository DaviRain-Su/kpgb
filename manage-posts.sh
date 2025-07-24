#!/bin/bash

# 文章管理脚本 - 选择性发布/取消发布/删除

echo "📚 KPGB 文章管理工具"
echo "=================="
echo ""

# 显示选项
echo "请选择操作："
echo "1) 取消发布测试文章（保留数据）"
echo "2) 删除所有测试文章（永久删除）"
echo "3) 只保留技术文章（隐藏个人笔记）"
echo "4) 自定义管理（交互式）"
echo "5) 查看当前发布状态"
echo "0) 退出"
echo ""

read -p "请输入选项 (0-5): " choice

case $choice in
    1)
        echo "🔒 取消发布测试文章..."
        # 测试相关的文章ID
        sqlite3 kpgb.db <<EOF
UPDATE posts SET published = 0 
WHERE storage_id IN (
    'QmVY3Aqhj94jdJLj4ituS8yNZ6WSx4hoA4NRpmivGeVx6w',
    'QmfMbK2SSPchbyhHz5pPYbdJZXgZJrkUpsE4ysEymDbruo',
    'QmWU4cL9EKYxzt5ds8uWsFS2btzRZ8GtE5SN5Q9GauktJe',
    'QmeJyWFN1hw53QNc581BT9SQ4dZVPZQm1WSyGTC1FPd5dk',
    'Qmf29xVBTuy2eUFHHtZQecxBdQDNxLEkWZeQULFS9QArst',
    'QmNxdQkHHpTfXW9MsHJ9kmZMHXukhBKUPeatNijy2iAr6n',
    'QmQmWyC1JXi269pT6J6Jnip9mP5aWHcquDfd7DFCjAYFo2',
    '49508d01325161cea625d5dfa585c4166c1736686b07e1c575769e65e1831539'
);
EOF
        echo "✅ 已取消发布所有测试文章"
        ;;
        
    2)
        echo "⚠️  警告：此操作将永久删除测试文章！"
        read -p "确认删除？(y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            cargo run -- delete QmVY3Aqhj94 --force
            cargo run -- delete QmfMbK2SSPc --force
            cargo run -- delete QmWU4cL9EKY --force
            cargo run -- delete QmeJyWFN1hw --force
            cargo run -- delete Qmf29xVBTuy --force
            cargo run -- delete QmNxdQkHHpT --force
            cargo run -- delete QmQmWyC1JXi --force
            cargo run -- delete 49508d01325 --force
            echo "✅ 已删除所有测试文章"
        else
            echo "❌ 取消删除"
        fi
        ;;
        
    3)
        echo "📝 只显示技术文章，隐藏个人笔记..."
        # 取消发布个人笔记
        sqlite3 kpgb.db <<EOF
UPDATE posts SET published = 0 
WHERE storage_id IN (
    'QmPEiHRYWUL85YYWPKVXCscJNL1Ka1u9hEv3BUFZLiXw1i',
    'QmYBGJGdy6aGmX6D5VvXhFFdsVdbkKzgsyVo3LnxGtUcut',
    'QmU243MrDk9VUwMcNSW7QPfPQRDTeJSdj6SYGcW5mEh7kY',
    'QmcBX5fvT3v6KjjKUmD2DK7NX81G4EKito1rNNEFs2papD'
);
EOF
        echo "✅ 已隐藏个人笔记"
        ;;
        
    4)
        echo "🔧 进入交互式管理模式..."
        echo ""
        # 列出所有已发布的文章
        sqlite3 -header -column kpgb.db "SELECT substr(storage_id, 1, 12) as ID, title, published FROM posts ORDER BY created_at DESC;"
        echo ""
        echo "使用以下命令管理文章："
        echo "  取消发布: cargo run -- edit <ID前缀> --unpublish"
        echo "  重新发布: cargo run -- publish <ID前缀>"
        echo "  删除文章: cargo run -- delete <ID前缀>"
        ;;
        
    5)
        echo "📊 当前文章发布状态："
        echo ""
        echo "已发布的文章："
        sqlite3 -column kpgb.db "SELECT substr(storage_id, 1, 12) as ID, title FROM posts WHERE published = 1 ORDER BY created_at DESC;"
        echo ""
        echo "未发布的文章："
        sqlite3 -column kpgb.db "SELECT substr(storage_id, 1, 12) as ID, title FROM posts WHERE published = 0 ORDER BY created_at DESC;"
        ;;
        
    0)
        echo "👋 退出"
        exit 0
        ;;
        
    *)
        echo "❌ 无效选项"
        ;;
esac

echo ""
echo "💡 提示：运行 'cargo run generate' 重新生成网站以应用更改"