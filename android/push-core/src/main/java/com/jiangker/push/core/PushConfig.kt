package com.jiangker.push.core

data class PushConfig(
    val xiaomi: XiaomiConfig? = null,
    val huawei: HuaweiConfig? = null,
    val oppo: OppoConfig? = null,
    val vivo: VivoConfig? = null,
    val honor: HonorConfig? = null,
    val meizu: MeizuConfig? = null,
    val server: PushServerConfig? = null,
)

data class PushServerConfig(
    val baseUrl: String,
    /** Push Hub 控制台中的应用 ID，设备注册时用于校验归属 */
    val hubAppId: String,
    /** 当前客户端平台标识（Android 为包名） */
    val packageName: String,
)

data class XiaomiConfig(
    val appId: String,
    val appKey: String,
)

data class HuaweiConfig(
    val appId: String,
)

data class OppoConfig(
    val appKey: String,
    val appSecret: String,
)

data class VivoConfig(
    val appId: String,
    val appKey: String,
)

data class HonorConfig(
    val appId: String,
)

data class MeizuConfig(
    val appId: String,
    val appKey: String,
)
