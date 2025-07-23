# 设置 Giscus 评论系统

Giscus 是一个基于 GitHub Discussions 的评论系统，完全免费且开源。

## 步骤 1：准备 GitHub 仓库

1. 确保你的仓库是**公开的**（Giscus 需要公开仓库）
2. 在仓库中启用 Discussions：
   - 进入仓库的 Settings
   - 在 Features 部分，勾选 "Discussions"

## 步骤 2：配置 Giscus

1. 访问 [giscus.app](https://giscus.app/zh-CN)
2. 按照页面指引配置：
   - **语言**：选择 `中文 (简体)`
   - **仓库**：输入 `davirain-su/kpgb`（或你的仓库名）
   - **页面 ↔️ discussion 映射关系**：选择 `pathname`
   - **Discussion 分类**：选择 `Announcements` 或 `General`
   - **特性**：
     - ✅ 启用反应表情
     - ✅ 评论框放在评论上方/下方（根据喜好）
     - ✅ 懒加载评论
   - **主题**：选择 `preferred_color_scheme`（跟随系统）

3. 配置完成后，页面会生成配置信息，包括：
   - `data-repo-id`
   - `data-category-id`

## 步骤 3：更新配置文件

将获取到的 ID 更新到 `site.toml`：

```toml
[giscus]
enabled = true
repo = "davirain-su/kpgb"              # 你的 GitHub 用户名/仓库名
repo_id = "R_kgDONa1b2Q"                # 从 giscus.app 获取
category = "Announcements"               # 选择的分类
category_id = "DIC_kwDONa1b2c4Ck3rN"    # 从 giscus.app 获取
mapping = "pathname"
reactions_enabled = true
emit_metadata = false
input_position = "bottom"                # top 或 bottom
theme = "preferred_color_scheme"         # 跟随系统主题
lang = "zh-CN"                          # 语言设置
```

## 步骤 4：生成并部署

```bash
# 生成静态网站
cargo run -- generate

# 提交并推送到 GitHub
git add .
git commit -m "Add Giscus comment system"
git push
```

## 步骤 5：验证

1. 等待 GitHub Pages 部署完成（约 1-2 分钟）
2. 访问你的博客文章页面
3. 滚动到底部应该能看到评论区
4. 第一次加载可能需要用户授权 GitHub

## 自定义主题

如果你想让 Giscus 更好地融入你的博客主题，可以：

1. 使用自定义主题 URL
2. 或选择预设主题：
   - `light` - 亮色主题
   - `dark` - 暗色主题
   - `dark_dimmed` - 暗色柔和
   - `transparent_dark` - 透明暗色
   - `preferred_color_scheme` - 跟随系统

## 常见问题

### 评论区不显示？
- 检查仓库是否公开
- 检查 Discussions 是否启用
- 检查 repo_id 和 category_id 是否正确
- 查看浏览器控制台是否有错误

### 如何管理评论？
- 所有评论都存储在 GitHub Discussions 中
- 可以在仓库的 Discussions 标签页管理评论
- 支持 Markdown 格式和表情反应

### 支持哪些浏览器？
- 所有现代浏览器（Chrome, Firefox, Safari, Edge）
- 需要 JavaScript 支持

## 其他评论系统选择

如果你不想使用 Giscus，还有其他选择：

1. **Utterances**：更轻量，基于 GitHub Issues
2. **Disqus**：功能丰富但有广告
3. **Waline**：支持匿名评论，需要自行部署
4. **Cusdis**：轻量级，注重隐私

每种都有各自的优缺点，选择适合你的即可。