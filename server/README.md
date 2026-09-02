# Push Hub Server

Rust 推送服务端，基于 Axum + PostgreSQL。

## 快速开始

```bash
# 1. 创建空库（需本机 PostgreSQL）
psql -U postgres -c "CREATE DATABASE push_hub WITH ENCODING 'UTF8' TEMPLATE template0;"

# 2. 配置连接串
cp .env.example .env
# 编辑 DATABASE_URL=postgres://user:pass@127.0.0.1:5432/push_hub

cargo run
```

服务启动时会通过 **sqlx** 自动执行 `migrations/` 下的数据库迁移（建表与索引）。

服务默认监听 `http://0.0.0.0:3000`。

## 数据库迁移

迁移文件位于 `migrations/`，按文件名顺序执行，版本记录在 `_sqlx_migrations` 表。

```bash
# 本地开发可选：安装 sqlx-cli 后手动执行
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run --database-url "$DATABASE_URL"
```

新增 schema 变更时，在 `migrations/` 下添加 `YYYYMMDDHHMMSS_description.sql`，勿再在 Repository 中写 `ALTER TABLE`。

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
# 新建空库（已有库可跳过）
psql -U postgres -c "CREATE DATABASE push_hub WITH ENCODING 'UTF8' TEMPLATE template0;"
```

表结构由服务端启动时自动迁移。

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
