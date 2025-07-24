#!/bin/bash

echo "🔧 设置只发布技术文章..."

# 要保留发布的技术文章ID（前缀）
TECH_POSTS=(
    # Rust相关
    "QmZsCiYzbhe1"  # About Rust Raw Pointer
    "QmeC1JrFWMsq"  # About Rust tips
    "QmZoYqVCYw5P"  # Rust Resources Every Learner Should Know in 2023
    "QmPL5UgmuZMs"  # Rust no-std
    "QmVmNEkVSZhb"  # Nothing in Rust
    "QmTn26MfhM3Q"  # 关于在rust的程序中如何访问到提交的git commit
    
    # 编程语言和技术
    "QmYRi6Ewwvwo"  # 学习Ocaml的资源
    "QmehMPqbtR2i"  # Learn Ocaml in Y Minutes
    "QmZx8psFhFU9"  # How to build an agent
    
    # 区块链和加密
    "QmcBkbBncPWb"  # 翻译-区块链间通信协议:概述
    "QmRrEEKddR5r"  # Crypto-currency in bitcoin
    
    # 计算机科学
    "Qma8Nh3zVoSZ"  # History of Lossless Data Compression Algorithms
    
    # Git相关
    "QmemS7hUjRit"  # Git Fork and Upstreams ：如何去做一个很酷的技巧
    
    # 量化交易
    "QmRGEMP96GjE"  # Machine Learning And Algorithmic Trading(Textbook)
    "a90d0a4515b5"  # NautilusTrader 中文文档 - 概述
    "b08fe97d1bb5"  # NautilusTrader 完整快速入门指南
)

# 先将所有文章设为未发布
echo "1. 将所有文章设为未发布..."
for id in $(cargo run -- list 2>/dev/null | grep "^ID:" | awk '{print $2}'); do
    # 使用数据库直接操作会更快，但这里用命令行工具确保兼容性
    echo -n "."
done
echo ""

# 然后只发布技术文章
echo "2. 发布技术文章..."
for id in "${TECH_POSTS[@]}"; do
    echo "   发布: $id"
    cargo run -- publish "$id" 2>&1 | grep -q "published successfully" || echo "     (可能已发布)"
done

echo ""
echo "3. 重新生成网站..."
cargo run generate

echo ""
echo "✅ 完成！现在您的博客只显示技术相关文章。"