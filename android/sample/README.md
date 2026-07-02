# Push Hub Sample App

Push Hub SDK 的本地调试示例，包名为 `com.jiangker.push.sample`（示例包名，请在你自己的厂商控制台注册对应应用）。

**依赖方式与第三方接入一致**：使用 `sample/libs/` 下的 AAR，不再 `project()` 引用各 SDK 模块。

## 快速运行

```bash
# 1. 准备 AAR（二选一）
#    a) 从 GitHub Release 下载，放入 sample/libs/，命名为 push-core-0.0.1.aar 等
#    b) 本地构建并同步：
cd android && ./gradlew syncPushHubAarsToSample

# 2. 配置推送参数（含厂商密钥，勿提交仓库）
cp sample/gradle.properties.example sample/gradle.properties
# 编辑 sample/gradle.properties，填入 Push Hub 与已开通厂商的参数

# 3. 启动服务端
cd ../server && cargo run

# 4. 构建并安装（默认 debug 签名）
cd ../android
./gradlew :sample:assembleDebug
```

第三方项目接入时，请参考 [Android 集成指南](../../docs/android-integration.md)，使用自己的包名与签名。

### Release 签名（可选）

示例工程**不包含** keystore。需要 release 包时，在本地生成签名并自行配置 `signingConfigs`（勿将 `.jks` 或密码提交到 Git）。

## 打开指定 Activity 调试

目标页：`DemoTargetActivity`（`singleTop`）。服务端按小米文档写入 `notify_effect=2` + `intent_uri`，冷启动走 `onCreate`，热启动（进程尚在）走 `onNewIntent`，页面会解析并展示 Intent 参数。

### 管理后台「发送推送」

点击行为与本次消息一起配置（模板不绑定点击）：

| 字段 | 值 |
|------|-----|
| 点击行为 | 打开页面 |
| Activity | `com.jiangker.push.sample.DemoTargetActivity`（全类名） |
| 页面参数 | 见下方键值 |

页面参数示例：

```json
{
  "order_id": "order-1001",
  "count": 3,
  "vip": true
}
```

选中任意模板、填好标题/变量与目标设备后发送到小米设备。

### 预期下发到小米的 intent_uri

```
intent:#Intent;component=com.jiangker.push.sample/.DemoTargetActivity;launchFlags=0x4000000;S.order_id=order-1001;i.count=3;B.vip=true;end
```

### 验证

1. **冷启动**：划掉 App → 点通知 → 进入「推送目标页」，顶部显示「冷启动 · onCreate」，下方列出参数。
2. **热启动**：先打开 App（或点首页「打开 DemoTargetActivity」）→ 回桌面保持进程 → 再点通知 → 显示「热启动 · onNewIntent」，参数刷新。
