# Giscus 评论系统设置步骤

## 快速设置

### 1. 启用 GitHub Discussions

首先确保你的仓库启用了 Discussions：

1. 访问你的仓库：https://github.com/davirain-su/kpgb
2. 点击 Settings（设置）
3. 向下滚动到 Features（功能）部分
4. 勾选 **Discussions**

### 2. 获取配置信息

访问 [giscus.app/zh-CN](https://giscus.app/zh-CN) 并按照以下步骤配置：

1. **选择语言**：中文（简体）

2. **仓库**：
   - 输入：`davirain-su/kpgb`
   - 系统会自动验证仓库是否满足条件

3. **页面 ↔️ discussion 映射关系**：
   - 选择：`pathname`（推荐）
   - 这样每篇文章会有独立的评论

4. **Discussion 分类**：
   - 选择：`Announcements` 或 `General`
   - 记住你选择的分类名

5. **特性**：
   - ✅ 启用反应
   - 选择评论框位置（顶部或底部）
   - ✅ 使用懒加载

6. **主题**：
   - 选择：`preferred_color_scheme`（跟随系统）

### 3. 复制生成的配置

配置完成后，页面底部会生成一段代码，找到其中的两个重要 ID：

```html
<script src="https://giscus.app/client.js"
        data-repo="davirain-su/kpgb"
        data-repo-id="R_kgDONa1b2Q"          <!-- 复制这个 -->
        data-category="Announcements"
        data-category-id="DIC_kwDONa1b2c4Ck3rN"  <!-- 复制这个 -->
        ...>
</script>
```

### 4. 更新 site.toml

编辑 `site.toml` 文件，填入获取到的 ID：

```toml
[giscus]
enabled = true
repo = "davirain-su/kpgb"
repo_id = "R_kgDONa1b2Q"                # 粘贴你的 repo-id
category = "Announcements"               
category_id = "DIC_kwDONa1b2c4Ck3rN"    # 粘贴你的 category-id
mapping = "pathname"
reactions_enabled = true
emit_metadata = false
input_position = "bottom"
theme = "preferred_color_scheme"
lang = "zh-CN"
```

### 5. 生成并部署

```bash
# 生成静态网站
cargo run -- generate

# 添加、提交并推送
git add .
git commit -m "Enable Giscus comments with proper IDs"
git push
```

### 6. 验证

1. 等待 GitHub Pages 部署完成（1-2分钟）
2. 访问任意文章页面
3. 滚动到底部查看评论区
4. 首次使用需要登录 GitHub 授权

## 注意事项

- 仓库必须是**公开的**
- 必须启用 Discussions
- repo_id 和 category_id 必须正确填写
- 评论会存储在 GitHub Discussions 中

## 故障排查

如果评论区不显示：

1. 检查浏览器控制台错误
2. 确认 repo_id 和 category_id 是否正确
3. 确认仓库是公开的且启用了 Discussions
4. 清除浏览器缓存后重试

需要帮助？在仓库创建 issue 或查看 [Giscus 文档](https://giscus.app/zh-CN)。