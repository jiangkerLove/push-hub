# Push Hub

可自托管的移动端推送基础设施，面向国内 Android（鸿蒙 / iOS 规划中）。统一设备身份、推送 API 与管理控制台，屏蔽小米 / 华为 / OPPO / vivo / 荣耀 / 魅族差异；**在线优先 WebSocket，离线自动降级厂商通知栏**。

## 特性

- **单一 SDK**：业务依赖 `:push`，`PushHub.init()` 接入；厂商模块按需添加，未集成则走在线通道
- **单一 device_id**：业务只认设备 ID，不感知底层 token / 平台
- **在线优先 + Outbox**：WS 实时下发；未送达可 HTTP 补拉，超时降级厂商 Push
- **模板与运营**：公信 / 私信、变量拼接、厂商合规字段、点击行为；Web 控制台含链路追踪与接入指南
- **自托管**：Rust Server + PostgreSQL + Vue Admin，推送 API Key 与管理端 JWT 分离

## 支持的平台

| 通道 | Android 模块 | 说明 |
|------|--------------|------|
| 小米 / 华为 / OPPO / vivo / 荣耀 / 魅族 | `:push-*` | 服务端对应 Provider |
| 在线 | `:push-core` 内置 | WebSocket + Outbox |
| HarmonyOS / iOS | — | 服务端字段已预留，客户端适配中 |

## 性能（实测摘要）

环境：**2C4G** 轻量云、单进程 Server。供选型参考，非 SLA。

| 同时在线 WS | Server 内存 |
|-------------|-------------|
| 0 | ≤ 10 MB |
| 1,000 | ≈ 140 MB |
| 5,000 | ≈ 700 MB |
| 10,000 | ≈ 1.37 GB |

1 万在线时向 **50** 台发消息，约 **1.1 s** 全部收到。粗算：`≈ 10 MB + N × 140 KB`。万级在线建议 **4C8G**。

## 技术栈与结构

| 组件 | 技术 |
|------|------|
| Server | Rust、Axum、Tokio、SQLx、PostgreSQL |
| Android SDK | Kotlin、各厂商官方 Push SDK |
| Admin | Vue 3、Vite、Element Plus |

```
push-hub/
├── android/     # SDK（:push 入口 + 各厂商模块 + sample）
├── server/      # 推送服务端
├── admin/       # 管理端
└── docs/        # 架构 / API / 集成文档
```

## 快速开始

**Server**

```bash
cd server && cp .env.example .env   # 配置 DATABASE_URL 等
cargo run                          # http://0.0.0.0:3000  →  GET /health
```

**Admin**

```bash
cd admin && npm install && npm run dev   # http://localhost:5173
```

首次打开创建管理员与应用；厂商凭证在控制台按应用配置。

**Android 示例**

```bash
cd android
cp sample/gradle.properties.example sample/gradle.properties
./gradlew :sample:assembleDebug
```

业务工程从 GitHub Releases 下载 AAR。详见 [Android 集成指南](docs/android-integration.md)。

## 部署

生产：`PostgreSQL` + Server + Admin（nginx 反代管理 API）。步骤概要：建库执行 `server/sql/schema.sql` → 启动 Server（`DATABASE_URL` / `JWT_SECRET`）→ 启动 Admin（`API_UPSTREAM`）。

完整 Docker / PostgreSQL 步骤见 [docs/deploy.md](docs/deploy.md)。

## 架构与 API

App 注册拿 `device_id` → 业务 `POST /api/v1/push` → 在线走 WS / Outbox，离线走厂商；超时未 ACK 则 Worker 降级。详见 [架构文档](docs/architecture.md)。

| 方法 | 路径 | 说明 |
|------|------|------|
| `POST` | `/api/v1/devices` | 注册设备（SDK） |
| `POST` | `/api/v1/push` | 发送推送（Push API Key） |
| `GET` | `/api/v1/online/ws` | 在线 WebSocket |
| `GET`/`POST` | `/api/v1/online/messages` | 轮询 / ACK |

完整说明：[服务端 API](docs/server-api.md) · [管理端 API](docs/admin-api.md)

## 安全

- 推送：应用级 `phk_…` API Key  
- 管理端：JWT；生产建议仅经 Admin 反代访问 admin API  
- 清单与风险：[docs/SECURITY.md](docs/SECURITY.md)

## 路线图

- [x] Android 六厂商 + 在线通道、Web 管理端、API Key / JWT、AAR 发布
- [ ] HarmonyOS / iOS SDK、CI 全量门禁、OpenAPI、厂商通道压测

## 许可证

[GPL-3.0](LICENSE)。SDK 若需更宽松许可证，请联系作者。

## 文档

- [架构设计](docs/architecture.md) · [部署](docs/deploy.md) · [安全](docs/SECURITY.md)
- [Android 集成](docs/android-integration.md) · [服务端 API](docs/server-api.md) · [管理端 API](docs/admin-api.md)
- [Server](server/README.md) · [Admin](admin/README.md)
