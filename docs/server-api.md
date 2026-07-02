# 服务端 API

Base URL 默认为 `http://localhost:3000`。本文档为**业务/SDK 公开 API**；设备列表、模板 CRUD 等管理操作见 [管理端 API](admin-api.md)（须 JWT，生产环境建议仅内网反代）。

所有 JSON API 的错误响应格式：

```json
{ "error": "错误描述" }
```

## 健康检查

### `GET /health`

```json
{ "status": "ok" }
```

---

## 设备

### `POST /api/v1/devices`

注册或更新设备（SDK 内部调用，也可手动测试）。

**请求体：**

```json
{
  "app_id": "uuid-from-push-hub-console",
  "device_id": "previously-bound-device-uuid",
  "package_name": "com.example.app",
  "platform": "xiaomi",
  "push_token": "vendor_push_token",
  "online_token": "online_polling_token"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `app_id` | string | 推荐 | Push Hub 控制台应用 ID，用于校验设备归属 |
| `device_id` | string | 可选 | 客户端已绑定的 device_id；厂商 token 刷新时传入，服务端只更新 push_token，保持身份稳定 |
| `package_name` | string | ✅ | 客户端平台标识（Android 包名 / iOS Bundle ID / 鸿蒙 Bundle Name） |
| `platform` | string | ✅ | 平台：`xiaomi` / `huawei` / `online` |
| `push_token` | string | ✅ | 厂商推送 token |
| `online_token` | string | 可选 | 在线推送 token |

**响应：** `Device` 对象

```json
{
  "id": "uuid",
  "app_id": "push-hub-app-uuid",
  "package_name": "com.example.app",
  "platform": "xiaomi",
  "push_token": "...",
  "online_token": "...",
  "last_online_at": "2026-01-01T00:00:00Z",
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

设备列表、模板管理等请使用管理端 API（`/api/v1/admin/...`，须 JWT），见 [管理端 API](admin-api.md)。

---

## 推送

### `POST /api/v1/push`

发送推送。**须鉴权**：请求头携带本应用在控制台「接入指南」中显示的 Push API Key。

```http
Authorization: Bearer phk_xxxxxxxx
```

或使用：

```http
X-Push-Hub-Api-Key: phk_xxxxxxxx
```

`app_id` 须与 API Key 所属应用一致；不传 `app_id` 时使用默认应用，Key 也必须属于该默认应用。

**请求体**（按模板类型选择字段；与管理端「发送推送」一致）：

**私信拼接模板**（模板预设标题/正文，含 `{{变量}}`）：

```json
{
  "app_id": "push-hub-app-uuid",
  "template_id": "template-uuid",
  "title_variables": {
    "order_id": "12345"
  },
  "body_variables": {
    "carrier": "顺丰",
    "tracking_no": "SF1234567890"
  },
  "payload": {
    "type": "order_shipped",
    "order_id": "12345"
  },
  "delivery_mode": "notification",
  "notify_id": 1001,
  "click_action": {
    "type": "open_page",
    "activity": "com.example.app.OrderDetailActivity",
    "params": { "order_id": "12345" }
  },
  "cache_until": "2026-07-13T08:00:00Z",
  "targets": {
    "device_ids": ["device-uuid-1", "device-uuid-2"]
  }
}
```

**公信模板 / 私信自由填写模板**（发送时填写标题与正文）：

```json
{
  "app_id": "push-hub-app-uuid",
  "template_id": "template-uuid",
  "title": "订单已发货",
  "body": "您的订单已发出，请注意查收。",
  "payload": {},
  "targets": {
    "device_ids": ["device-uuid-1"]
  }
}
```

**无模板直推**（省略 `template_id`，须传 `title` + `body`）：

```json
{
  "app_id": "push-hub-app-uuid",
  "title": "订单已发货",
  "body": "您的订单已发出，请注意查收。",
  "targets": {
    "device_ids": ["device-uuid-1"]
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `app_id` | string | 推荐 | Push Hub 应用 ID（UUID）；须与 API Key 所属应用一致 |
| `template_id` | string | 条件 | 模板 ID；模板推送时必填。省略时可仅用 `title` + `body` 直推 |
| `title` | string | 条件 | 公信模板、私信自由填写模板、或无模板直推时必填 |
| `body` | string | 条件 | 同上 |
| `title_variables` | object · Record&lt;string, string&gt; | 条件 | 私信拼接模板：标题中 `{{key}}` 的取值，如 `{ "order_id": "12345" }` |
| `body_variables` | object · Record&lt;string, string&gt; | 条件 | 私信拼接模板：正文中 `{{key}}` 的取值，如 `{ "carrier": "顺丰" }` |
| `payload` | object | 可选 | 业务自定义 JSON，默认 `{}`，透传给客户端 |
| `delivery_mode` | string · `"notification"` \| `"data"` | 可选 | 投递模式，默认 `notification` |
| `notify_id` | integer | 可选 | 通知栏 ID（0~2147483647，JSON 整数非字符串）；相同 ID 覆盖旧通知 |
| `click_action` | object | 可选 | 点击行为：`type`（string 枚举）、`activity`/`url`（string）、`params`（object） |
| `cache_until` | string · ISO 8601 | 可选 | 在线消息缓存截止时间，如 `2026-07-13T08:00:00Z` |
| `targets` | object | 必填 | 推送目标容器 |
| `targets.device_ids` | string[] | 二选一 | 按 device_id 发送，如 `["uuid-1"]` |
| `targets.push_tokens` | string[] | 二选一 | 按 token 直发（需配合 `platform`） |
| `targets.platform` | string | 条件 | 使用 `push_tokens` 时必填，如 `xiaomi`、`huawei` |
| `channels` | object | 可选 | 无模板直推时的厂商通道配置，键为平台名 |

**click_action.type 取值：**

| 值 | 说明 |
|----|------|
| `open_app` | 打开 Launcher Activity |
| `open_page` | 打开指定 Activity（`activity` 须为全类名，如 `com.example.app.OrderDetailActivity`；可选 `params`） |
| `open_web` | 打开网页（需 `url` 字段） |

**响应：**

```json
{
  "total": 2,
  "success": 2,
  "failed": 0,
  "platforms": [
    {
      "platform": "online",
      "success": 1,
      "failed": 0,
      "message_id": null
    },
    {
      "platform": "xiaomi",
      "success": 1,
      "failed": 0,
      "message_id": "msg-xxx"
    }
  ]
}
```

---

## 在线推送

### `GET /api/v1/online/ws?push_token={token}`

WebSocket 长连接，用于实时接收在线推送。SDK 内部使用。

**服务端 → 客户端消息：**

```json
{
  "type": "message",
  "id": "outbox-id",
  "title": "标题",
  "body": "内容",
  "payload": {},
  "delivery_mode": "notification",
  "notify_id": 1001,
  "created_at": "2026-01-01T00:00:00Z"
}
```

`notify_id` 可选；客户端展示系统通知时优先使用该 ID，相同 ID 的新消息会覆盖旧通知。

**客户端 → 服务端消息：**

```json
{ "type": "ack", "ids": ["outbox-id-1"] }
{ "type": "ping" }
```

### `GET /api/v1/online/messages?push_token={token}&limit=20`

HTTP 轮询拉取未读消息（WebSocket 断连时的兜底）。

### `POST /api/v1/online/messages`

ACK 已读消息。

**请求体：**

```json
{
  "push_token": "online_token",
  "ids": ["outbox-id-1", "outbox-id-2"]
}
```

**响应：**

```json
{ "acked": 2 }
```

---

## 环境变量

详见 `server/.env.example`：

| 变量 | 说明 |
|------|------|
| `SERVER_HOST` | 监听地址（默认 `0.0.0.0`） |
| `SERVER_PORT` | 监听端口（默认 `3000`） |
| `DATABASE_URL` | PostgreSQL 连接串（如 `postgres://postgres:postgres@127.0.0.1:5432/push_hub`） |
| `JWT_SECRET` | 管理端 JWT 密钥 |
| `ONLINE_PUSH_FALLBACK_SECS` | 新建应用的默认在线降级秒数 |
| `ONLINE_MESSAGE_CACHE_SECS` | 新建应用的默认消息缓存秒数 |

首次启动不会自动创建管理员或默认应用；请在管理端完成初始化。厂商推送凭证（小米、华为、OPPO 等）在**管理控制台 → 应用配置**中按应用填写，无需写入 `.env`。
