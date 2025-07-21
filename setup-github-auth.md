# 解决 GitHub 认证问题

GitHub 从 2021 年 8 月起不再支持密码认证。有两种解决方案：

## 方案 1：使用 Personal Access Token（推荐）

### 步骤 1：创建 Personal Access Token
1. 登录 GitHub
2. 点击右上角头像 → Settings
3. 左侧菜单最下方 → Developer settings
4. Personal access tokens → Tokens (classic)
5. Generate new token → Generate new token (classic)
6. 设置：
   - Note: `KPGB Deploy`
   - Expiration: 选择合适的过期时间
   - 勾选权限：
     - ✅ repo (全部)
     - ✅ workflow
7. 点击 "Generate token"
8. **重要**：复制生成的 token（只显示一次！）

### 步骤 2：使用 Token 替代密码
```bash
# 当提示输入密码时，粘贴你的 token（不是密码！）
Username: DaviRain-Su
Password: ghp_xxxxxxxxxxxxxxxxxxxx  # 这里粘贴你的 token
```

### 步骤 3：保存认证信息（可选）
```bash
# macOS：使用钥匙串保存
git config --global credential.helper osxkeychain

# 或者临时保存（15分钟）
git config --global credential.helper cache
```

## 方案 2：使用 SSH（长期使用推荐）

### 步骤 1：生成 SSH 密钥
```bash
# 检查是否已有 SSH 密钥
ls -la ~/.ssh

# 如果没有，生成新的
ssh-keygen -t ed25519 -C "your_email@example.com"
# 一路回车使用默认设置
```

### 步骤 2：添加 SSH 密钥到 GitHub
```bash
# 复制公钥
cat ~/.ssh/id_ed25519.pub
# 或者 macOS 直接复制到剪贴板
pbcopy < ~/.ssh/id_ed25519.pub
```

1. GitHub → Settings → SSH and GPG keys
2. New SSH key
3. Title: `KPGB Deploy Key`
4. Key: 粘贴复制的内容
5. Add SSH key

### 步骤 3：切换到 SSH URL
```bash
# 查看当前远程仓库
git remote -v

# 切换到 SSH
git remote set-url origin git@github.com:DaviRain-Su/kpgb.git

# 测试连接
ssh -T git@github.com
```

## 快速解决方案（立即部署）

如果你急于部署，可以：

### 选项 1：手动创建 gh-pages 分支
```bash
# 1. 在 GitHub 网页上创建空的 gh-pages 分支

# 2. 本地创建并推送
git checkout -b gh-pages
rm -rf *
cp -r public/* .
git add .
git commit -m "Deploy static site"

# 3. 使用 GitHub Desktop 或其他 GUI 工具推送
```

### 选项 2：使用 GitHub Actions（自动部署）
创建 `.github/workflows/deploy.yml`：

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [ main, master ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Generate static site
      run: |
        cargo run -- generate
        touch public/.nojekyll
    
    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./public
```

## 推荐步骤

1. **最快**：创建 Personal Access Token，立即使用
2. **最安全**：设置 SSH，长期使用
3. **最自动**：配置 GitHub Actions，推送即部署

---

💡 **提示**：Token 要妥善保管，可以设置较短的过期时间（如 30 天），需要时再创建新的。