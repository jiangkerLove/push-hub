package com.jiangker.push.core

import android.content.Context
import android.content.pm.PackageManager
import android.os.Bundle

/**
 * 从合并后的 Application meta-data 读取 Push Hub 与厂商凭证。
 *
 * 推荐在 App 模块 `defaultConfig.manifestPlaceholders` 中统一配置占位符，
 * 各 push-* 模块 Manifest 会将其写入固定 meta-data 键名；初始化时无需在代码里重复传入。
 */
object PushConfigReader {
    object Meta {
        const val SERVER = "com.jiangker.push.server"
        const val APP_ID = "com.jiangker.push.app_id"

        const val XIAOMI_APP_ID = "com.jiangker.push.xiaomi.app_id"
        const val XIAOMI_APP_KEY = "com.jiangker.push.xiaomi.app_key"

        const val HUAWEI_APP_ID = "com.jiangker.push.huawei.app_id"
        const val OPPO_APP_KEY = "com.jiangker.push.oppo.app_key"
        const val OPPO_APP_SECRET = "com.jiangker.push.oppo.app_secret"
        const val VIVO_APP_ID = "com.jiangker.push.vivo.app_id"
        const val VIVO_APP_KEY = "com.jiangker.push.vivo.app_key"
        const val HONOR_APP_ID = "com.jiangker.push.honor.app_id"
        const val MEIZU_APP_ID = "com.jiangker.push.meizu.app_id"
        const val MEIZU_APP_KEY = "com.jiangker.push.meizu.app_key"

        const val HMS_APP_ID = "com.huawei.hms.client.appid"
        const val VIVO_PUSH_APP_ID = "com.vivo.push.app_id"
        const val VIVO_PUSH_API_KEY = "com.vivo.push.api_key"
        const val HONOR_PUSH_APP_ID = "com.hihonor.push.app_id"
    }

    fun loadMeta(context: Context): Bundle? = runCatching {
        context.packageManager.getApplicationInfo(
            context.packageName,
            PackageManager.GET_META_DATA,
        ).metaData
    }.getOrNull()

    fun getString(meta: Bundle?, key: String): String? {
        if (meta == null) return null
        val raw = meta.get(key)?.toString()?.trim().orEmpty()
        if (raw.isEmpty() || looksLikeUnresolvedPlaceholder(raw)) return null
        return raw
    }

    fun resolveServer(meta: Bundle?, override: String?): String? =
        override?.trim()?.takeIf { it.isNotEmpty() } ?: getString(meta, Meta.SERVER)

    fun resolveAppId(meta: Bundle?, override: String?): String? =
        override?.trim()?.takeIf { it.isNotEmpty() } ?: getString(meta, Meta.APP_ID)

    fun resolveXiaomi(meta: Bundle?, appId: String?, appKey: String?): XiaomiConfig? {
        val resolvedId = appId?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.XIAOMI_APP_ID)
        val resolvedKey = appKey?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.XIAOMI_APP_KEY)
        if (resolvedId.isNullOrBlank() || resolvedKey.isNullOrBlank()) return null
        return XiaomiConfig(resolvedId, resolvedKey)
    }

    fun resolveHuawei(meta: Bundle?, appId: String?): HuaweiConfig? {
        val resolvedId = appId?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.HUAWEI_APP_ID)
            ?: getString(meta, Meta.HMS_APP_ID)?.let(::parseHuaweiAppId)
        if (resolvedId.isNullOrBlank()) return null
        return HuaweiConfig(resolvedId)
    }

    fun resolveOppo(meta: Bundle?, appKey: String?, appSecret: String?): OppoConfig? {
        val resolvedKey = appKey?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.OPPO_APP_KEY)
        val resolvedSecret = appSecret?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.OPPO_APP_SECRET)
        if (resolvedKey.isNullOrBlank() || resolvedSecret.isNullOrBlank()) return null
        return OppoConfig(resolvedKey, resolvedSecret)
    }

    fun resolveVivo(meta: Bundle?, appId: String?, appKey: String?): VivoConfig? {
        val resolvedId = appId?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.VIVO_APP_ID)
            ?: getString(meta, Meta.VIVO_PUSH_APP_ID)
        val resolvedKey = appKey?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.VIVO_APP_KEY)
            ?: getString(meta, Meta.VIVO_PUSH_API_KEY)
        if (resolvedId.isNullOrBlank() || resolvedKey.isNullOrBlank()) return null
        return VivoConfig(resolvedId, resolvedKey)
    }

    fun resolveHonor(meta: Bundle?, appId: String?): HonorConfig? {
        val resolvedId = appId?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.HONOR_APP_ID)
            ?: getString(meta, Meta.HONOR_PUSH_APP_ID)
        if (resolvedId.isNullOrBlank()) return null
        return HonorConfig(resolvedId)
    }

    fun resolveMeizu(meta: Bundle?, appId: String?, appKey: String?): MeizuConfig? {
        val resolvedId = appId?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.MEIZU_APP_ID)
        val resolvedKey = appKey?.trim()?.takeIf { it.isNotEmpty() }
            ?: getString(meta, Meta.MEIZU_APP_KEY)
        if (resolvedId.isNullOrBlank() || resolvedKey.isNullOrBlank()) return null
        return MeizuConfig(resolvedId, resolvedKey)
    }

    private fun parseHuaweiAppId(raw: String): String {
        val trimmed = raw.trim()
        return trimmed.removePrefix("appid=").trim().ifBlank { trimmed }
    }

    private fun looksLikeUnresolvedPlaceholder(value: String): Boolean =
        value.startsWith("\${") && value.endsWith("}")
}
