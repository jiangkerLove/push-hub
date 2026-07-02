# Android SDK 发布说明

## 产物

| 文件 | 说明 |
|------|------|
| `push-core-<version>.aar` | 核心实现 |
| `push-<version>.aar` | 统一入口（依赖 push-core） |
| `push-xiaomi-<version>.aar` 等 | 各厂商适配（按需；**已内嵌**厂商 SDK，华为为 HMS push+base） |

版本号见 `android/VERSION`，或由 tag `android-v<version>` / Actions 输入覆盖。

> 华为：从华为 Maven **fat-aar embed** push 及其 HMS 传递依赖（含 `opendevice` 的 `HmsInstanceId`/aaid）。push 与 opendevice 有重复 string 资源，构建时在 `gradle.properties` 启用 `android.disableResourceValidation=true`。

## 维护者：更新厂商 AAR

官方 SDK 不提交 git（见 `android/.gitignore`；已入库的历史文件除外）。按各模块 `build.gradle.kts` 里 `embed(files("libs/....aar"))` 的文件名，下载后放入对应 `libs/` 目录后执行 `assemblePushHubAars`。

## 本地打 AAR

```bash
cd android
./gradlew assemblePushHubAars "-PpushHubVersion=0.0.1"
```

PowerShell 下请给 `-P...` 加引号。

产物在 `*/build/outputs/aar/*-release.aar`。

## CI

Workflow：`.github/workflows/android-aar.yml`

触发方式：

1. 推送 tag：`android-v0.0.1`
2. Actions 手动运行 **Android AAR**

发布到仓库 **GitHub Releases**（各模块独立 AAR 附件，按需下载）：

- `push-core-<version>.aar`、`push-<version>.aar`（必选）
- `push-xiaomi-<version>.aar` 等厂商模块（按需）

消费方从 Releases 下载，见 [android-integration.md](../docs/android-integration.md)。

本地如需发布到 GitHub Packages，可执行 `./gradlew publishReleaseToGitHubPackages`（需配置 `GITHUB_TOKEN`）。
