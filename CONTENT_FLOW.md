# KPGB 内容流程详解

## 一、新文章发布流程

### 1. 创建文章
```bash
# 方式1：交互式创建
cargo run -- new --title "我的新文章" --author "作者名"
# 然后输入内容，按 Ctrl+D 结束

# 方式2：从文件导入
cargo run -- new --title "我的新文章" --author "作者名" --content article.md
```

### 2. 内容处理流程
```
用户输入 → 内容处理 → 存储决策 → 数据库索引
```

#### 详细步骤：

1. **内容接收**
   - 接收标题、作者、内容等信息
   - 生成唯一ID (UUID)
   - 生成URL slug
   - 记录创建时间

2. **内容哈希计算**
   ```rust
   // 计算内容的SHA256哈希
   let content_hash = sha256(content);
   ```
   - 用于内容去重
   - 确保相同内容不会重复存储

3. **存储决策**
   ```rust
   // 检查是否已存在相同内容
   if database.content_exists(content_hash) {
       // 复用已有存储ID
       storage_id = database.get_storage_id(content_hash);
   } else {
       // 存储到配置的后端（IPFS/Local/GitHub）
       storage_id = storage.store(content, metadata).await?;
   }
   ```

4. **IPFS存储过程**（如果使用IPFS后端）
   ```rust
   // 1. 连接到本地IPFS节点
   let client = IpfsClient::new("http://localhost:5001");
   
   // 2. 添加内容到IPFS
   let response = client.add(content).await?;
   let cid = response.hash; // 获得CID，如：QmXxx...
   
   // 3. Pin内容确保不被垃圾回收
   client.pin_add(&cid).await?;
   ```

5. **数据库索引**
   ```sql
   INSERT INTO posts (
       id, storage_id, title, slug, content, 
       author, content_hash, published, created_at
   ) VALUES (?, ?, ?, ?, ?, ?, ?, false, CURRENT_TIMESTAMP);
   
   -- 更新全文搜索索引
   INSERT INTO posts_fts (title, content, author, tags) 
   VALUES (?, ?, ?, ?);
   ```

### 3. 发布文章
```bash
# 将草稿状态改为已发布
cargo run -- publish <storage-id>
```

这会更新数据库中的 `published` 字段和 `published_at` 时间戳。

## 二、内容在IPFS网络的传播

### 1. IPFS内容寻址原理
```
内容 → SHA256哈希 → CID (Content Identifier)
"Hello" → QmWATWQ7fVPP2EFGu71UkfnqhYXDYH566qy47CnJDgvs8u
```

### 2. 内容传播过程
```
本地节点 → DHT网络 → 其他节点请求 → P2P传输
```

1. **本地节点发布**
   - 内容存储在本地IPFS仓库
   - CID被广播到DHT（分布式哈希表）
   - 其他节点可通过CID请求内容

2. **节点间同步**
   - 当其他节点请求CID时，拥有该内容的节点会响应
   - 内容通过P2P协议传输
   - 接收节点验证内容哈希是否匹配CID

3. **Pin机制**
   ```bash
   # 确保内容不被清理
   ipfs pin add QmXxx...
   
   # 查看已pin的内容
   ipfs pin ls
   ```

## 三、新设备同步方案

### 方案1：导出/导入索引（推荐）

#### 在原设备上导出
```bash
# 1. 导出数据库
cp kpgb.db kpgb-export.db

# 2. 导出已pin的CID列表
sqlite3 kpgb.db "SELECT storage_id FROM posts WHERE storage_id LIKE 'Qm%'" > cids.txt

# 3. 打包
tar -czf kpgb-backup.tar.gz kpgb-export.db cids.txt
```

#### 在新设备上导入
```bash
# 1. 解压备份
tar -xzf kpgb-backup.tar.gz

# 2. 恢复数据库
cp kpgb-export.db kpgb.db

# 3. Pin所有CID到本地IPFS
while read cid; do
    ipfs pin add "$cid"
done < cids.txt

# 4. 启动服务
cargo run -- serve
```

### 方案2：IPNS发布索引（自动同步）

#### 发布端设置
```bash
# 1. 生成索引文件
cargo run -- export-index > index.json

# 2. 发布到IPFS
INDEX_CID=$(ipfs add -q index.json)

# 3. 更新IPNS指向最新索引
ipfs name publish --key=kpgb-index $INDEX_CID
```

#### 同步端设置
```bash
# 1. 获取IPNS地址对应的最新索引
IPNS_ADDR="QmYourIPNSAddress"
INDEX_CID=$(ipfs name resolve $IPNS_ADDR)

# 2. 获取索引文件
ipfs get $INDEX_CID -o index.json

# 3. 导入索引
cargo run -- import-index index.json
```

### 方案3：分布式索引协议（未来功能）

```yaml
# 索引共享协议设计
protocol: kpgb/index/1.0.0
operations:
  - ANNOUNCE: 广播拥有的内容CID列表
  - REQUEST: 请求特定作者的内容索引
  - SYNC: 同步两个节点的索引差异
  - SUBSCRIBE: 订阅某个作者的更新
```

## 四、完整工作流示例

### 场景：在设备A创建内容，在设备B访问

#### 设备A（内容创建者）
```bash
# 1. 确保IPFS运行
ipfs daemon &

# 2. 创建新文章
cargo run -- new --title "分布式博客的未来" --author "DaviRain"

# 3. 查看存储结果
cargo run -- list
# 输出: Storage ID: QmaBcDeFgHiJkLmNoPqRsTuVwXyZ...

# 4. 发布文章
cargo run -- publish QmaBcDeFgHiJkLmNoPqRsTuVwXyZ...

# 5. 生成静态网站
cargo run -- generate

# 6. 部署到GitHub Pages
./scripts/deploy.sh
```

#### 设备B（内容消费者）
```bash
# 方式1：通过IPFS网关访问
curl https://ipfs.io/ipfs/QmaBcDeFgHiJkLmNoPqRsTuVwXyZ...

# 方式2：同步到本地
# 1. 获取设备A的索引（通过安全渠道共享）
scp deviceA:~/kpgb/kpgb.db ./

# 2. Pin所有内容到本地
sqlite3 kpgb.db "SELECT storage_id FROM posts" | while read cid; do
    ipfs pin add "$cid"
done

# 3. 启动本地服务
cargo run -- serve
```

## 五、数据完整性保证

### 1. 内容验证
- 每个CID都是内容的加密哈希
- IPFS自动验证下载内容的完整性
- 无法篡改已发布的内容

### 2. 索引备份策略
```bash
# 定期备份脚本
#!/bin/bash
DATE=$(date +%Y%m%d)
BACKUP_DIR="backups"

# 备份数据库
cp kpgb.db "$BACKUP_DIR/kpgb-$DATE.db"

# 导出CID列表
sqlite3 kpgb.db "SELECT storage_id, title FROM posts" > "$BACKUP_DIR/posts-$DATE.csv"

# 创建IPFS快照
ipfs add -r ~/.ipfs/blocks > "$BACKUP_DIR/ipfs-blocks-$DATE.txt"
```

### 3. 多节点冗余
```yaml
# 推荐的节点配置
nodes:
  - primary: 
      location: "家庭服务器"
      always_on: true
      auto_pin: true
  - backup:
      location: "云VPS"
      mirror: true
  - mobile:
      location: "笔记本"
      selective_sync: true
```

## 六、故障恢复

### 场景1：本地IPFS节点数据丢失
```bash
# 从数据库恢复所有CID
sqlite3 kpgb.db "SELECT storage_id FROM posts WHERE storage_id LIKE 'Qm%'" | \
while read cid; do
    echo "Recovering $cid..."
    ipfs get "$cid" || echo "Failed to recover $cid"
done
```

### 场景2：数据库损坏
```bash
# 从IPFS重建索引（需要已知CID列表）
for cid in $(cat known-cids.txt); do
    content=$(ipfs cat "$cid")
    # 解析内容并重建数据库记录
    cargo run -- import-from-ipfs "$cid"
done
```

## 七、最佳实践

1. **定期备份**
   - 每周备份数据库
   - 导出CID列表
   - 测试恢复流程

2. **多节点部署**
   - 至少保持2个IPFS节点在线
   - 使用IPFS Cluster实现自动同步

3. **内容组织**
   - 使用有意义的标题和标签
   - 定期清理草稿
   - 保持索引精简

4. **安全考虑**
   - 不要在内容中包含敏感信息
   - IPFS内容是公开的
   - 考虑加密私密内容

## 八、未来改进方向

1. **自动同步机制**
   - 基于IPNS的索引自动发现
   - P2P索引交换协议
   - 实时内容订阅

2. **增强功能**
   - 内容加密选项
   - 多作者协作
   - 版本控制
   - 评论系统

3. **工具改进**
   - GUI客户端
   - 移动应用
   - 浏览器扩展
   - 一键部署脚本