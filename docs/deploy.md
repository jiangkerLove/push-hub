# 部署

生产环境建议：`PostgreSQL` + `push-hub-server` + `push-hub-admin`。Server 依赖 PostgreSQL；Admin 通过 nginx 反代 `/api/v1/admin/` 访问 Server。

```
浏览器 ──► Admin(:8080 / nginx)
              │  /api/v1/admin/*
              ▼
           Server(:3000)  ──►  /api/v1/push、/devices …（业务 API，可对公网）
              │
              ▼
         PostgreSQL(:5432)
```

## 1. PostgreSQL

### 方案 A：复用已有实例

```bash
psql -U postgres -c "CREATE DATABASE push_hub WITH ENCODING 'UTF8' TEMPLATE template0;"
psql -U postgres -d push_hub -f server/sql/schema.sql
```

连接串示例：

```text
DATABASE_URL=postgres://用户名:密码@主机:5432/push_hub
```

可选：单独建用户并只授权 `push_hub`：

```sql
CREATE USER push_hub WITH PASSWORD '换成强密码';
GRANT ALL PRIVILEGES ON DATABASE push_hub TO push_hub;
\c push_hub
GRANT ALL ON SCHEMA public TO push_hub;
GRANT ALL ON ALL TABLES IN SCHEMA public TO push_hub;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO push_hub;
```

### 方案 B：Docker 新建

```bash
docker network create push-hub

docker run -d --name push-hub-postgres \
  --network push-hub \
  -e POSTGRES_USER=push_hub \
  -e POSTGRES_PASSWORD=换成强密码 \
  -e POSTGRES_DB=push_hub \
  -v push-hub-pgdata:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres:16

docker exec -i push-hub-postgres \
  psql -U push_hub -d push_hub < server/sql/schema.sql
```

同网络内 Server：

```text
DATABASE_URL=postgres://push_hub:换成强密码@push-hub-postgres:5432/push_hub
```

## 2. Server（Docker）

```bash
cd server
cargo build --release --target x86_64-unknown-linux-musl

docker build -t push-hub-server .
docker run -d --name push-hub-server \
  --network push-hub \
  -p 3000:3000 \
  -e SERVER_HOST=0.0.0.0 \
  -e SERVER_PORT=3000 \
  -e DATABASE_URL=postgres://push_hub:换成强密码@push-hub-postgres:5432/push_hub \
  -e JWT_SECRET=换成足够长的随机串 \
  push-hub-server
```

| 环境变量 | 说明 |
|---------|------|
| `DATABASE_URL` | PostgreSQL 连接串（必填） |
| `JWT_SECRET` | 管理端 JWT 签名密钥，生产务必更换 |
| `SERVER_HOST` / `SERVER_PORT` | 默认 `0.0.0.0:3000` |

复用宿主机 PostgreSQL（不在 Docker 网络内）时，Linux 可加：

```bash
--add-host=host.docker.internal:host-gateway \
-e DATABASE_URL=postgres://用户名:密码@host.docker.internal:5432/push_hub
```

健康检查：`GET http://<host>:3000/health`。首次启动不自动创建管理员，打开 Admin 按引导创建。

另见 [server/README.md](../server/README.md#docker-部署)。

## 3. Admin（Docker）

```bash
cd admin
docker build -t push-hub-admin .
docker run -d --name push-hub-admin \
  --network push-hub \
  -p 8080:80 \
  -e API_UPSTREAM=http://push-hub-server:3000 \
  push-hub-admin
```

访问 http://\<host\>:8080 。`API_UPSTREAM` 不要带尾斜杠。详见 [admin/README.md](../admin/README.md#docker-部署)。
