# Push Hub Server

Rust 推送服务端，基于 Axum + PostgreSQL。

## 快速开始

```bash
# 1. 建库并表结构（需本机 PostgreSQL）
psql -U postgres -c "CREATE DATABASE push_hub WITH ENCODING 'UTF8' TEMPLATE template0;"
psql -U postgres -d push_hub -f sql/schema.sql

# 2. 配置连接串
cp .env.example .env
# 编辑 DATABASE_URL=postgres://user:pass@127.0.0.1:5432/push_hub

cargo run
```

服务默认监听 `http://0.0.0.0:3000`。创建客户端：先建库 `push_hub`，再连接该库执行 `sql/schema.sql`。

## 开发

```bash
cargo fmt
cargo clippy
cargo test
```

API 文档见 [docs/server-api.md](../docs/server-api.md)。

## Docker 部署

依赖 PostgreSQL。可复用已有实例，或单独起一个库；完整步骤（含 Admin）见 [docs/deploy.md](../docs/deploy.md)。

### 准备数据库

```bash
# 新建库（已有库可跳过）
psql -U postgres -c "CREATE DATABASE push_hub WITH ENCODING 'UTF8' TEMPLATE template0;"

# 初始化表结构
psql -U postgres -d push_hub -f sql/schema.sql
```

### 构建并运行

```bash
# 先交叉编译 musl 发布包
cargo build --release --target x86_64-unknown-linux-musl

docker build -t push-hub-server .
docker run -d --name push-hub-server \
  -p 3000:3000 \
  -e DATABASE_URL=postgres://用户名:密码@host.docker.internal:5432/push_hub \
  -e JWT_SECRET=换成足够长的随机串 \
  --add-host=host.docker.internal:host-gateway \
  push-hub-server
```

首次启动不会自动创建管理员；打开管理端按引导创建账号，登录后再创建应用。

若 PostgreSQL 也在 Docker 同一网络，把 `DATABASE_URL` 主机改成容器名（如 `push-hub-postgres`），并去掉 `--add-host`。
