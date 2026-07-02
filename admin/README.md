# Push Hub Admin

Vue 3 管理端，用于管理应用、推送配置、模板和设备。

## 功能

- 管理员登录（JWT）
- 应用管理（CRUD、设为默认）
- 推送配置（小米/华为密钥、在线降级超时）
- 推送模板管理
- 设备列表查看
- 测试推送发送

## 开发

先启动服务端：

```bash
cd ../server
cp .env.example .env
cargo run
```

再启动管理端：

```bash
cd admin
npm install
npm run dev
```

本地开发时 Vite 默认将 `/api` 代理到 `http://127.0.0.1:3000`。若需指向其他环境，复制 `.env.development.example` 为 `.env.development` 并设置 `VITE_API_PROXY_TARGET`。

访问 http://localhost:5173

首次打开管理端时，若尚未创建管理员，会进入「创建管理员」引导；创建并登录后，再在应用列表中创建第一个应用。

## 构建

```bash
npm run build
```

构建产物在 `dist/` 目录。

## Docker 部署

管理端镜像会在容器内完成 `npm run build`，再用 nginx 托管静态资源，并**仅**反代 `/api/v1/admin/` 到推送服务端（不暴露业务 API）。

```bash
cd admin
docker build -t push-hub-admin .
docker run --rm -p 8080:80 \
  -e API_UPSTREAM=http://host.docker.internal:3000 \
  push-hub-admin
```

访问 http://localhost:8080 。

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `API_UPSTREAM` | `http://host.docker.internal:3000` | 推送服务端地址，**不要**带尾斜杠 |

若 admin 与 server 在同一 Docker 网络：

```bash
docker run --rm -p 8080:80 \
  --network push-hub \
  -e API_UPSTREAM=http://push-hub-server:3000 \
  push-hub-admin
```

Linux 上若要用 `host.docker.internal`，需额外加 `--add-host=host.docker.internal:host-gateway`。

与 PostgreSQL、Server 一起部署的完整步骤见 [docs/deploy.md](../docs/deploy.md)。
