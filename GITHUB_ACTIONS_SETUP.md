# GitHub Actions 自动部署设置

我已经为你创建了两个 GitHub Actions 工作流：

## 1. 自动部署到 GitHub Pages (`deploy.yml`)

这个工作流会在你推送代码到 `main` 或 `master` 分支时自动触发：
- 构建 Rust 项目
- 生成静态网站
- 自动部署到 GitHub Pages

## 2. 构建和测试 (`build-and-test.yml`)

这个工作流会在每次推送和 PR 时运行：
- 检查代码格式
- 运行 clippy 检查
- 运行测试
- 构建项目

## 设置步骤

### 1. 启用 GitHub Pages

1. 访问仓库设置：https://github.com/DaviRain-Su/kpgb/settings/pages
2. 在 "Build and deployment" 下
3. Source: 选择 "GitHub Actions"
4. 保存

### 2. 推送代码触发部署

```bash
# 添加所有文件
git add .

# 提交（包括 .github 文件夹）
git commit -m "Add GitHub Actions for automated deployment"

# 推送到 GitHub
git push origin main
```

### 3. 查看部署状态

1. 访问：https://github.com/DaviRain-Su/kpgb/actions
2. 你会看到 "Deploy to GitHub Pages" 工作流正在运行
3. 等待完成（通常需要 2-3 分钟）
4. 完成后访问：https://DaviRain-Su.github.io/kpgb

## 工作流程说明

### 自动部署流程：
1. **触发**：推送到 main/master 分支
2. **构建**：
   - 安装 Rust
   - 缓存依赖加速构建
   - 创建生产配置（自动使用你的 GitHub 用户名）
   - 生成静态网站
3. **部署**：
   - 上传到 GitHub Pages
   - 自动发布

### 优势：
- ✅ 无需本地部署
- ✅ 无需处理认证问题
- ✅ 自动使用正确的 URL
- ✅ 每次推送自动更新博客

## 自定义配置

如果需要修改博客标题、作者等信息，编辑 `.github/workflows/deploy.yml` 中的这部分：

```yaml
- name: Create production config
  run: |
    cat > site.production.toml << EOL
    title = "My IPFS Blog"  # 修改这里
    description = "A decentralized blog powered by IPFS"  # 修改这里
    author = "Your Name"  # 修改这里
    # ... 其他配置
    EOL
```

## 手动触发部署

如果需要手动触发部署：
1. 访问：https://github.com/DaviRain-Su/kpgb/actions
2. 点击 "Deploy to GitHub Pages"
3. 点击 "Run workflow"
4. 选择分支并运行

## 故障排除

### 如果部署失败：
1. 检查 Actions 日志
2. 确保仓库设置中启用了 GitHub Pages
3. 确保选择了 "GitHub Actions" 作为源

### 如果页面 404：
1. 等待几分钟（首次部署可能需要时间）
2. 检查是否正确启用了 GitHub Pages
3. 确认 URL 是否正确

## 下一步

1. 推送这些文件到 GitHub
2. 等待 Actions 完成
3. 访问你的博客：https://DaviRain-Su.github.io/kpgb

之后每次你：
- 创建新文章：`cargo run -- new ...`
- 推送到 GitHub：`git push`
- 博客会自动更新！

无需再手动部署！🎉