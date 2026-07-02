# 架构设计

## 总体架构

Push Hub 由三个层次组成：

```
┌─────────────────────────────────────────────────────────────┐
│                        业务层                                │
│  业务 App / 后端服务  ──► POST /api/v1/push                  │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                     push-hub server                          │
│  ┌──────────┐  ┌─────────────┐  ┌────────────────────────┐  │
│  │ routes   │→ │ PushService │→ │ PushProvider (trait)   │  │
│  └──────────┘  └─────────────┘  │  ├─ XiaomiPushProvider │  │
│                                  │  ├─ HuaweiPushProvider │  │
│                                  │  └─ OnlinePushProvider │  │
│  ┌──────────┐                    └────────────────────────┘  │
│  │ db       │  devices / templates / outbox                   │
│  └──────────┘                                                  │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                     Android SDK                              │
│  PushHub → PushVendorLoader → PushManager → 单厂商 PushProvider │
└─────────────────────────────────────────────────────────────┘
```

## 服务端分层

| 层 | 目录 | 职责 |
|----|------|------|
| 路由层 | `routes/` | HTTP/WebSocket 入口，参数校验，响应序列化 |
| 业务层 | `push/service.rs` | 推送编排：目标解析、模板渲染、通道选择 |
| Provider 层 | `push/*.rs` | 各推送通道的具体实现（trait `PushProvider`） |
| 数据层 | `db/` | Repository trait + PostgreSQL 实现 |
| 模型层 | `models/` | 请求/响应/领域模型 |

### PushProvider 抽象

所有推送通道实现统一的 `PushProvider` trait：

```rust
#[async_trait]
pub trait PushProvider: Send + Sync {
    fn platform(&self) -> &'static str;
    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult>;
}
```

当前注册的 Provider：

| platform | 实现 | 说明 |
|----------|------|------|
| `xiaomi` | `XiaomiPushProvider` | 小米官方 REST API |
| `huawei` | `HuaweiPushProvider` | 华为 Push Kit REST API |
| `online` | `OnlinePushProvider` | 内部 Outbox + WebSocket |

`PushHub` 作为 Provider 注册中心，在启动时按配置动态注册。

## Android SDK 分层

| 模块 | 职责 |
|------|------|
| `:push` | 对外 API（`PushHub`），仅依赖 `:push-core`；厂商模块由业务方按需引入 |
| `:push-core` | 初始化编排、设备注册、在线连接、消息分发 |
| `:push-xiaomi` | 小米 SDK 封装 |
| `:push-huawei` | 华为 HMS Push 封装 |
| `:push-oppo` / `:push-vivo` / `:push-honor` / `:push-meizu` | 其余厂商 SDK 封装 |

### 初始化策略

`PushHub.init()` 先识别设备厂商，再仅创建并初始化匹配的厂商 `PushProvider`（无匹配或未配置时走在线推送）。

### 设备身份模型

每个物理设备在服务端只有一条 device 记录：

| 字段 | 说明 |
|------|------|
| `id` | 服务端生成的唯一 device_id，业务推送时使用 |
| `platform` | 厂商平台（`xiaomi` / `huawei` / `online`） |
| `push_token` | 厂商推送 token |
| `online_token` | 在线推送轮询/WebSocket token（内部字段） |
| `last_online_at` | 最近一次在线心跳时间 |

业务方通过 `PushHub.getDeviceId()` 获取 `device_id`，无需关心底层 token。SDK 会本地持久化已绑定的 `device_id`；厂商 pushId 刷新时携带旧 id 重新注册，服务端只更新 `push_token`，保持设备身份稳定。

## 推送通道选择

服务端 `PushService::dispatch_target` 按以下逻辑选择通道：

```
厂商密钥已配置，且推送消息已填该平台通道（channel_id / category）？
  ├─ 否 → OnlinePushProvider（仅在线，不尝试厂商）
  └─ 是 → 优先在线（Outbox + WebSocket）
           ├─ WebSocket 已送达 → 结束
           └─ 未送达 / 入队失败 → 降级厂商离线推送
```

推送通道可选：模板/直推未配置对应平台通道时默认只走在线。填了通道才会在在线未送达时降级厂商。荣耀无通道字段，有密钥即可走厂商。

### Outbox 机制

在线推送不直接经 WebSocket 发送，而是：

1. 写入 `outbox` 表
2. 通过 `OnlinePushHub` 广播到已连接的 WebSocket
3. 客户端 ACK 后标记为已送达
4. 超时未 ACK → fallback Worker 触发厂商推送

这样即使 WebSocket 断连，客户端仍可通过 HTTP 轮询拉取未读消息。

## 模板系统

推送内容通过模板管理，支持：

- **变量替换**：模板 title/body 中使用 `{{variable}}` 占位符
- **厂商通道配置**：每个模板可单独配置小米 `channel_id`、华为 / vivo / OPPO `category`、魅族 `msg_type`（PUBLIC / PRIVATE）；OPPO 非 IM 私信与我方拼接模板一对一绑定（`private_template_id`）：发送时其它厂商自动拼 `{{变量}}`，OPPO 转成官方模板 ID + 参数；荣耀无需通道配置（有推送密钥即可）
- **在线缓存天数**：模板只配置默认缓存天数；发送时按发送时刻换算截止时间，也可在发送请求中用 `cache_until` 覆盖
- **点击行为**：在发送请求中配置 `open_app` / `open_page` / `open_web`（可带页面参数），与具体消息内容相关，不绑定模板；`open_page` 的 `activity` 须为全类名，各厂商与在线通道统一按全类名跳转

发送推送时传入 `template_id` + 变量 / 直接文案 + `click_action`，服务端渲染后分发给 Provider。

## 数据库 Schema（PostgreSQL）

完整建库脚本见 `server/sql/schema.sql`。主要表：

| 表 | 用途 |
|----|------|
| `apps` | 应用与厂商凭证 |
| `devices` | 设备注册信息 |
| `push_templates` | 推送模板 |
| `push_channels` | 厂商通道/分类配置 |
| `push_outbox` | 在线推送消息队列 |
| `push_jobs` / `push_job_targets` / `push_job_events` | 推送链路追踪 |
| `admin_users` | 管理端账号 |

Repository 通过 trait 抽象；建库脚本见 `server/sql/schema.sql`，需手动执行。

## 管理端预留

后续管理端（Web Admin）将通过独立的 Admin API 接入，与服务端共用 `db` 层：

```
admin/web  ──►  /api/v1/admin/*  ──►  db / PushService（管理端 JWT 鉴权）
                      ↑
                  JWT / API Key 鉴权
```

当前模板与设备 API 已具备 CRUD 能力，管理端可直接复用，后续增加鉴权中间件即可。
