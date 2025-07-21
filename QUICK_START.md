# KPGB 快速开始指南

## 5 分钟上手

### 1. 快速体验（无需 IPFS）

```bash
# 创建第一篇文章
cargo run -- new --title "Hello World" --author "我"

# 查看文章列表
cargo run -- list

# 启动 Web 界面
cargo run -- serve

# 访问 http://localhost:3000
```

### 2. 完整 IPFS 体验

#### 步骤 1: 启动 IPFS
```bash
# 安装 IPFS
brew install ipfs  # macOS
# 或访问 https://ipfs.io/docs/install/

# 初始化并启动
ipfs init
ipfs daemon  # 在新终端窗口运行
```

#### 步骤 2: 配置环境
```bash
# 创建 .env 文件
echo "IPFS_API_URL=http://localhost:5001" > .env
```

#### 步骤 3: 创建并发布文章
```bash
# 创建文章
cargo run -- new --title "我的 IPFS 博客" --author "张三"

# 发布文章（获取文章 ID 后）
cargo run -- publish <文章ID>

# 文章将永久存储在 IPFS 网络中！
```

### 3. 生成静态网站

```bash
# 初始化配置
cargo run -- init

# 生成网站
cargo run -- generate

# 查看生成的网站
open ./public/index.html  # macOS
# 或 xdg-open ./public/index.html  # Linux
```

## 常用命令速查

| 功能 | 命令 | 说明 |
|------|------|------|
| 新建文章 | `cargo run -- new -t "标题" -a "作者"` | 创建新文章 |
| 列出文章 | `cargo run -- list` | 查看所有文章 |
| 发布文章 | `cargo run -- publish <ID>` | 发布到 IPFS |
| 搜索文章 | `cargo run -- search "关键词"` | 全文搜索 |
| 阅读文章 | `cargo run -- read <ID>` | 查看文章内容 |
| 生成网站 | `cargo run -- generate` | 生成静态 HTML |
| 启动服务 | `cargo run -- serve` | 启动 Web 界面 |

## 实际使用示例

### 场景 1: 写技术博客
```bash
# 1. 创建文章文件
echo "# Rust 异步编程入门

本文介绍 Rust 的异步编程模型...

## Tokio 简介
..." > rust-async.md

# 2. 导入到系统
cargo run -- new -t "Rust 异步编程入门" -a "技术小哥" -c rust-async.md

# 3. 发布
cargo run -- publish <返回的ID>
```

### 场景 2: 批量导入现有博客
```bash
# 创建脚本导入多个 Markdown 文件
for file in ./old-blog/*.md; do
    title=$(head -n 1 "$file" | sed 's/# //')
    cargo run -- new -t "$title" -a "我" -c "$file"
done
```

### 场景 3: 搭建个人网站
```bash
# 1. 生成静态网站
cargo run -- generate

# 2. 部署到 GitHub Pages
cd public
git init
git add .
git commit -m "Initial blog"
git remote add origin https://github.com/你的用户名/你的用户名.github.io.git
git push -u origin main

# 3. 访问 https://你的用户名.github.io
```

## 小贴士

1. **备份重要**: 定期备份 `kpgb.db` 数据库文件
2. **IPFS 网关**: 如果本地 IPFS 不可用，可以通过公共网关访问内容
3. **性能优化**: 使用 `--release` 编译以获得最佳性能
4. **自定义模板**: 编辑 `templates/` 目录下的文件来自定义网站外观

## 故障排除

**问题**: IPFS 连接失败
```bash
# 检查 IPFS 是否运行
ipfs id

# 检查 API 端口
curl http://localhost:5001/api/v0/id
```

**问题**: 端口被占用
```bash
# 使用其他端口
cargo run -- serve --port 8080
```

**问题**: 数据库错误
```bash
# 重新初始化数据库
rm kpgb.db
cargo run -- list  # 会自动创建新数据库
```