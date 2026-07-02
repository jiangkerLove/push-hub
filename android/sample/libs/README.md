# Sample 使用的 Push Hub AAR

本目录放置 **版本化** 的 release AAR，文件名须与 `android/VERSION` 一致，例如 `push-core-0.0.1.aar`。

## 方式一：从 Release 下载

在 [GitHub Releases](https://github.com/jiangkerLove/push-hub/releases) 下载所需模块 AAR，放入此目录并重命名为 `模块名-版本.aar`。

至少需要：

- `push-core-<version>.aar`
- `push-<version>.aar`
- 已开通的厂商模块（如 `push-huawei-<version>.aar`）

## 方式二：本地构建后同步

```bash
cd android
./gradlew syncPushHubAarsToSample
```

会将各模块 `assembleRelease` 产物复制到本目录。

> `*.aar` 已 gitignore，勿提交到仓库。
