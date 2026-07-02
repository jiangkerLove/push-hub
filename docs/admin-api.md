# 管理端 API

> **内部接口**：仅供 Push Hub 管理控制台使用，**不对外开放**，也不属于业务集成 API。  
> 生产环境请通过 Admin 控制台（nginx 反代）访问。

Base URL（经 Admin 反代）：`http://<admin-host>/api/v1/admin`  
本地开发：`http://localhost:3000/api/v1/admin`

除登录外，所有接口需在 Header 中携带：

```
Authorization: Bearer <token>
```

## 登录与初始化

### `GET /admin/bootstrap`

查询是否需要首次创建管理员（无需鉴权）。

```json
{ "needs_setup": true }
```

### `POST /admin/setup`

仅在尚无管理员时可用，创建首个管理员并返回登录 token。

```json
{
  "username": "admin",
  "password": "your-password"
}
```

**响应：**

```json
{
  "token": "eyJ...",
  "username": "admin"
}
```

### `POST /admin/login`

**请求体：**

```json
{
  "username": "admin",
  "password": "your-password"
}
```

**响应：**

```json
{
  "token": "eyJ...",
  "username": "admin"
}
```

## 当前用户

### `GET /admin/me`

```json
{
  "username": "admin",
  "is_owner": true,
  "display_time_zone": "Asia/Shanghai"
}
```

`display_time_zone` 由主账号设置，子账号只读共用；未设置时为 `null`（前端默认按 UTC 展示，主账号首次登录会用浏览器时区初始化）。

### `PUT /admin/settings/display-timezone`

仅主账号可调用。在管理端「账号管理 → 系统设置」中配置。

```json
{ "display_time_zone": "Asia/Shanghai" }
```

**响应：** 同 `GET /admin/me`。

## 账号管理

### `GET /admin/users`

仅主账号。返回全部管理员账号列表（不含密码）。

### `POST /admin/users`

仅主账号。创建子账号。

```json
{ "username": "ops", "password": "your-password" }
```

### `DELETE /admin/users/{id}`

仅主账号。删除子账号（不可删除主账号或当前登录账号）。

### `PUT /admin/me/password`

任意已登录账号修改自己的密码。成功后当前登录态立即失效，需用新密码重新登录。

```json
{
  "current_password": "old-password",
  "new_password": "new-password"
}
```

**响应：**

```json
{ "ok": true, "require_relogin": true }
```

### `PUT /admin/me/username`

修改当前登录账号的用户名，返回新 token。

```json
{ "username": "new-name" }
```

**响应：**

```json
{
  "token": "eyJ...",
  "username": "new-name",
  "is_owner": false,
  "display_time_zone": "Asia/Shanghai"
}
```

### `PUT /admin/users/{id}/password`

仅主账号。重置指定子账号密码（不可重置主账号，主账号请用 `/admin/me/password`）。

```json
{ "new_password": "new-password" }
```

## 应用管理

### `GET /admin/apps`

应用列表（不含密钥明文摘要）。

### `POST /admin/apps`

创建应用。

```json
{
  "name": "示例应用",
  "package_name": "com.jiangker.push.sample",
  "description": "本地调试应用",
  "xiaomi_app_secret": "...",
  "huawei_app_id": "...",
  "huawei_app_secret": "...",
  "online_push_fallback_secs": 90
}
```

### `GET /admin/apps/{id}`

获取应用详情（含推送密钥，仅管理端使用）。

### `PUT /admin/apps/{id}`

更新应用信息与推送配置。

### `DELETE /admin/apps/{id}`

删除应用（默认应用不可删除）。

### `POST /admin/apps/{id}/default`

设为默认应用。

## 模板管理（按应用）

### `GET /admin/apps/{id}/templates`

### `POST /admin/apps/{id}/templates`

### `PUT /admin/templates/{id}`

### `DELETE /admin/templates/{id}`

## 设备列表

### `GET /admin/apps/{id}/devices?limit=50&offset=0`

按应用 `package_name` 过滤设备。

## 发送测试推送

### `POST /admin/apps/{id}/push`

请求体同公开推送 API `POST /api/v1/push`，使用当前应用的推送配置。

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `JWT_SECRET` | — | JWT 签名密钥 |

首次启动不会自动创建管理员或默认应用。打开管理端后按引导创建账号，登录后再创建应用。
