# Android 集成指南

## 前置条件

- Android Studio Ladybug 或更高版本
- minSdk 23，compileSdk 36
- 已在对应厂商开放平台创建应用并获取推送参数
- push-hub 服务端已启动（见 [README](../README.md) 快速开始）

## 1. 添加依赖（GitHub Releases）

Push Hub Android SDK 以 AAR 发布到仓库 **Releases**；打 `android-v*` tag 或手动跑 Actions 后，在对应 Release **按需下载各模块 AAR 附件**。

### 1.1 下载 AAR

1. 打开仓库 [Releases](https://github.com/jiangkerLove/push-hub/releases)
2. 按需下载附件（无需下载全部）：
   - **必选**：`push-core-<version>.aar`、`push-<version>.aar`
   - **按需**：`push-xiaomi-<version>.aar`、`push-huawei-<version>.aar` 等
3. 将 AAR 复制到业务 App 的 `libs/` 目录

### 1.2 引入模块

在 App 模块 `build.gradle.kts`：

```kotlin
dependencies {
    implementation(files("libs/push-core-0.1.0.aar"))
    implementation(files("libs/push-0.1.0.aar"))
    // 只添加已开通的厂商模块（厂商官方 SDK 已内嵌）
    implementation(files("libs/push-xiaomi-0.1.0.aar"))
    implementation(files("libs/push-huawei-0.1.0.aar"))
}
```

> 小米 / OPPO / vivo / 荣耀 / 魅族 / 华为模块的 AAR **已内嵌**对应厂商官方 SDK（华为含 HMS push 闭包）。第三方项目用 `files("libs/*.aar")` 接入时**无需**配置厂商 Maven 仓库、`agconnect-services.json` 或荣耀 `mcs-services.json`，只需在下一步配置 `manifestPlaceholders`。

## 2. 配置 Manifest 占位符

Push Hub 与厂商参数通过 **Manifest meta-data** 注入，App 只需在 `defaultConfig.manifestPlaceholders` 配置一次，初始化时 SDK 自动读取，**无需在代码里重复传入**。

```kotlin
android {
    defaultConfig {
        manifestPlaceholders += mapOf(
            // Push Hub
            "PUSH_HUB_SERVER" to "https://your-push-hub-server.com",
            "PUSH_HUB_APP_ID" to "your-push-hub-app-id",
            // 小米
            "XIAOMI_APP_ID" to "your_xiaomi_app_id",
            "XIAOMI_APP_KEY" to "your_xiaomi_app_key",
            "XIAOMI_CHANNEL_ID" to "your_xiaomi_channel_id",
            // 华为
            "HUAWEI_APP_ID" to "your_huawei_app_id",
            // OPPO
            "OPPO_APP_KEY" to "your_oppo_app_key",
            "OPPO_APP_SECRET" to "your_oppo_app_secret",
            // vivo
            "VIVO_APP_ID" to "your_vivo_app_id",
            "VIVO_APP_KEY" to "your_vivo_app_key",
            // 荣耀
            "HONOR_APP_ID" to "your_honor_app_id",
            // 魅族
            "MEIZU_APP_ID" to "your_meizu_app_id",
            "MEIZU_APP_KEY" to "your_meizu_app_key",
        )
    }
}
```

建议**始终保留全部键名**；未引入的厂商模块对应值可留空字符串。控制台「接入指南」会按本应用已配置通道填入真实参数。

## 3. 运行时通道选择

无需 Product Flavor 分渠道打包。`PushHub.init()` 会先识别设备厂商，再只初始化匹配且已在 Manifest 中配置了凭证的通道：

- 华为机 → 仅初始化华为 HMS 通道
- 小米机 → 仅初始化小米通道
- 未匹配到已配置的厂商通道 → 走在线推送（WebSocket）

## 4. 初始化 SDK

在 `Application.onCreate()` 中：

```kotlin
class MyApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        PushHub.init(
            context = this,
            config = PushHubConfig.Builder()
                .messageService(MyPushService::class.java)
                .build(this),
        )
    }
}
```

`build(this)` 会从 Manifest 自动加载 Push Hub 服务端地址、应用 ID 与各厂商凭证。如需覆盖某个值，仍可使用 `.server()` / `.app()` / `.xiaomi()` 等显式设置。

### PushHubConfig 参数

| 方法 | 必填 | 说明 |
|------|------|------|
| `messageService(class)` | ✅ | 继承 `PushMessageService` 的消息接收 Service |
| `build(context)` | ✅ | 从 Manifest 补全其余配置 |
| `server(baseUrl)` | 可选 | 覆盖 Manifest 中的服务端地址 |
| `app(appId)` | 可选 | 覆盖 Manifest 中的 Push Hub 应用 ID |
| `xiaomi` / `huawei` / … | 可选 | 覆盖对应厂商 Manifest 凭证 |
| `registrationListener` | 可选 | 设备注册成功回调 |

## 5. 实现消息接收 Service

```kotlin
class MyPushService : PushMessageService() {
    override fun onNewToken(vendor: PushVendor, token: String) {
        // 厂商 token 更新（SDK 会自动重新注册到服务端）
    }

    override fun onDeviceRegistered(vendor: PushVendor, deviceId: String) {
        // 获得服务端 device_id，可用于上报业务后端
    }

    override fun onMessageReceived(vendor: PushVendor, message: PushMessage) {
        // 收到推送消息（在线/离线通道统一回调）
    }

    override fun onError(vendor: PushVendor, errorMessage: String?) {
        // 推送错误
    }
}
```

在 `AndroidManifest.xml` 中注册：

```xml
<service
    android:name=".MyPushService"
    android:exported="false" />
```

## 6. 获取 Device ID

```kotlin
// 推荐：获取唯一设备 ID，发送推送时使用
val deviceId = PushHub.getDeviceId()
```

将 `deviceId` 上报到你的业务后端，发送推送时在 `targets.device_ids` 中指定。

## 7. ProGuard

Release 构建需合并 Push Hub 的 ProGuard 规则：

```kotlin
buildTypes {
    release {
        proguardFiles(
            getDefaultProguardFile("proguard-android-optimize.txt"),
            "proguard-rules.pro",
            "../common/proguard-rules.pro",
        )
    }
}
```

## 示例 App

仓库 `android/sample/` 提供了完整可运行的示例：

```bash
cd android
cp sample/gradle.properties.example sample/gradle.properties
# 编辑 sample/gradle.properties（勿提交）

./gradlew :sample:assembleDebug
```

示例包名：`com.jiangker.push.sample`。厂商密钥写在 `sample/gradle.properties`（已 gitignore），通过 `manifestPlaceholders` 注入；`SampleApplication` 初始化时无需在代码里重复传厂商参数。

## 常见问题

### 在线推送不生效

1. 确认服务端 `ONLINE_PUSH_FALLBACK_SECS` 配置合理（默认 90 秒）
2. 确认 App 在前台或后台存活，WebSocket 连接正常
3. 检查 Logcat 中 `PushHub-Registrar` / `PushHub-Online` 标签

### 厂商推送不到

1. 确认服务端厂商密钥正确
2. 确认包名与厂商控制台绑定一致
3. 确认 `manifestPlaceholders` 中对应厂商键名与值正确

### 某厂商设备上无厂商推送

1. App 是否已添加对应厂商模块 AAR
2. `manifestPlaceholders` 是否包含该厂商凭证
3. Logcat 中 `PushHub-VendorLoader` 是否输出 `resolved provider`
