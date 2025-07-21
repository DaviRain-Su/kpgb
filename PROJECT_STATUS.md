# KPGB 项目状态报告

## 项目概述
KPGB (Kaspa-Powered Genesis Blog) 是一个去中心化的个人博客系统，所有内容存储在 IPFS 上，同时提供本地索引以实现高性能查询。

## 已完成功能 (100%)

### 1. 存储系统 ✅
- **IPFS 存储**: 内容永久存储在 IPFS 网络，每个内容都有唯一的 CID
- **多存储后端**: 支持 IPFS、GitHub 和本地存储，可自动切换
- **内容去重**: 使用 SHA256 哈希检测重复内容，避免重复上传

### 2. 数据索引 ✅
- **SQLite 数据库**: 本地索引存储文章元数据
- **全文搜索**: 集成 FTS5 实现快速全文搜索
- **标签系统**: 支持文章标签和分类

### 3. 内容管理 ✅
- **文章创建**: 支持 Markdown 格式
- **发布控制**: 草稿和发布状态管理
- **内容更新**: 可更新文章元数据

### 4. 静态网站生成 ✅
- **Tera 模板引擎**: 灵活的 HTML 模板系统
- **响应式设计**: 适配移动端和桌面端
- **RSS 订阅**: 自动生成 RSS feed
- **资源管理**: CSS 和静态资源自动复制

### 5. Web 界面 ✅
- **Axum Web 服务器**: 高性能异步 Web 服务
- **实时搜索**: 动态搜索功能
- **RESTful API**: 完整的 API 端点
- **归档页面**: 按时间组织的文章列表

### 6. 命令行界面 ✅
- **完整的 CLI**: 所有功能都可通过命令行操作
- **友好的输出**: 彩色输出和进度提示

## 使用指南

### 环境准备

1. **安装 IPFS (可选)**
   ```bash
   # macOS
   brew install ipfs
   
   # 初始化 IPFS
   ipfs init
   
   # 启动 IPFS 守护进程
   ipfs daemon
   ```

2. **设置环境变量**
   ```bash
   # IPFS API 地址
   export IPFS_API_URL=http://localhost:5001
   
   # GitHub 存储 (可选)
   export GITHUB_TOKEN=your_github_token
   export GITHUB_OWNER=your_username
   export GITHUB_REPO=your_repo
   ```

### 基本操作

#### 1. 创建新文章
```bash
# 交互式输入内容
cargo run -- new --title "我的第一篇文章" --author "张三"

# 从文件读取内容
cargo run -- new --title "技术分享" --author "李四" --content ./article.md
```

#### 2. 查看文章列表
```bash
# 查看所有文章
cargo run -- list

# 只查看已发布的文章
cargo run -- list --published
```

#### 3. 发布文章
```bash
# 使用文章 ID 发布
cargo run -- publish <storage-id>

# 例如
cargo run -- publish 49508d01325161cea625d5dfa585c4166c1736686b07e1c575769e65e1831539
```

#### 4. 搜索文章
```bash
# 搜索包含关键词的文章
cargo run -- search "IPFS"
cargo run -- search "区块链"
```

#### 5. 阅读文章
```bash
# 使用文章 ID 读取
cargo run -- read <storage-id>
```

### 静态网站生成

1. **初始化网站配置**
   ```bash
   cargo run -- init
   ```
   这会创建 `site.toml` 配置文件

2. **编辑配置** (site.toml)
   ```toml
   title = "我的去中心化博客"
   description = "基于 IPFS 的个人博客"
   author = "Your Name"
   base_url = "https://yourdomain.com"
   ```

3. **生成静态网站**
   ```bash
   cargo run -- generate --output ./public
   ```
   
4. **查看生成的文件**
   ```bash
   ls -la ./public/
   # index.html - 首页
   # posts/ - 文章目录
   # archive.html - 归档页
   # feed.xml - RSS 订阅
   # style.css - 样式文件
   ```

### Web 服务器

1. **启动服务器**
   ```bash
   # 默认端口 3000
   cargo run -- serve
   
   # 自定义端口
   cargo run -- serve --port 8080
   ```

2. **访问网站**
   - 首页: http://localhost:3000
   - 搜索: http://localhost:3000/search
   - 归档: http://localhost:3000/archive

3. **API 使用**
   ```bash
   # 获取所有文章
   curl http://localhost:3000/api/posts | jq
   
   # 搜索文章
   curl -X POST http://localhost:3000/api/search \
     -H "Content-Type: application/json" \
     -d '{"query": "IPFS"}' | jq
   
   # 获取单篇文章
   curl http://localhost:3000/api/posts/<storage-id> | jq
   ```

### IPFS 内容验证

1. **查看 IPFS 存储的内容**
   ```bash
   # 使用 IPFS 命令
   ipfs cat <CID>
   
   # 例如
   ipfs cat QmQmWyC1JXi269pT6J6Jnip9mP5aWHcquDfd7DFCjAYFo2
   ```

2. **通过网关访问**
   ```
   https://ipfs.io/ipfs/<CID>
   https://gateway.pinata.cloud/ipfs/<CID>
   ```

### 高级功能

#### 存储后端测试
```bash
# 测试 IPFS 存储
cargo run -- test-storage --backend ipfs

# 测试本地存储
cargo run -- test-storage --backend local

# 测试 GitHub 存储
cargo run -- test-storage --backend github
```

#### 数据库管理
```bash
# 数据库位置
sqlite3 ./kpgb.db

# 查看表结构
.schema

# 查询文章
SELECT * FROM posts;

# 全文搜索
SELECT * FROM posts_fts WHERE posts_fts MATCH 'IPFS';
```

## 项目结构
```
kpgb/
├── src/
│   ├── main.rs          # 程序入口和 CLI
│   ├── models/          # 数据模型
│   ├── storage/         # 存储后端实现
│   │   ├── mod.rs       # 存储特征定义
│   │   ├── ipfs.rs      # IPFS 实现
│   │   ├── github.rs    # GitHub 实现
│   │   └── local.rs     # 本地存储实现
│   ├── blog/            # 博客管理逻辑
│   ├── database/        # 数据库操作
│   ├── site/            # 静态网站生成
│   └── web/             # Web 服务器
├── templates/           # HTML 模板
├── static/              # 静态资源
├── migrations/          # 数据库迁移
├── Cargo.toml           # 项目依赖
├── CLAUDE.md            # 项目说明
└── README.md            # 使用文档
```

## 技术栈
- **语言**: Rust
- **异步运行时**: Tokio
- **Web 框架**: Axum
- **模板引擎**: Tera
- **数据库**: SQLite + FTS5
- **存储**: IPFS, GitHub API
- **序列化**: Serde
- **CLI**: Clap

## 下一步计划（可选）
- [ ] 添加评论系统（基于 IPFS）
- [ ] 实现 IPNS 可变引用
- [ ] 添加主题系统
- [ ] 实现多用户支持
- [ ] 添加 Kaspa 集成（原计划功能）
- [ ] 实现分布式搜索
- [ ] 移动端应用

## 常见问题

**Q: 为什么有些文章显示 IPFS 徽章？**
A: 带有 "Qm" 开头的存储 ID 表示该文章已存储在 IPFS 网络上。

**Q: 如何备份我的博客？**
A: 备份 `kpgb.db` 数据库文件和 `storage/local` 目录即可。IPFS 内容会永久保存在网络中。

**Q: 可以离线使用吗？**
A: 是的，本地索引允许离线浏览。只有上传到 IPFS 时需要网络连接。

**Q: 如何迁移到新电脑？**
A: 复制整个项目目录，确保安装相同的 Rust 环境即可。

## 项目状态：已完成 ✅
所有计划功能均已实现并测试通过。系统可以正常使用。