package com.jiangker.push.sample

import android.content.Intent
import android.os.Bundle

/**
 * 解析推送点击带进 Activity 的 Intent extras。
 *
 * 小米等厂商通过 intent_uri 写入基本类型参数（S./i./B. 等），
 * 冷启动走 [Activity.onCreate]，热启动（singleTop + CLEAR_TOP）走 [Activity.onNewIntent]。
 */
object PushIntentExtras {
    /** 过滤系统/框架字段，只保留业务参数。 */
    private val ignoredKeys = setOf(
        "android.intent.extra.COMPONENT_NAME",
        "profile",
    )

    fun parse(intent: Intent?): Map<String, String> {
        if (intent == null) return emptyMap()
        val extras = intent.extras ?: return emptyMap()
        return parseBundle(extras)
    }

    fun parseBundle(extras: Bundle): Map<String, String> {
        val result = linkedMapOf<String, String>()
        for (key in extras.keySet()) {
            if (key.isNullOrBlank() || key in ignoredKeys || key.startsWith("android.")) continue
            @Suppress("DEPRECATION")
            val value = extras.get(key) ?: continue
            result[key] = formatValue(value)
        }
        return result
    }

    fun describeLaunch(coldStart: Boolean, params: Map<String, String>): String {
        val mode = if (coldStart) "冷启动（onCreate）" else "热启动（onNewIntent）"
        if (params.isEmpty()) return "$mode · 无业务参数"
        val body = params.entries.joinToString("\n") { (k, v) -> "  $k = $v" }
        return "$mode\n$body"
    }

    private fun formatValue(value: Any): String = when (value) {
        is String -> value
        is Boolean, is Byte, is Short, is Int, is Long, is Float, is Double -> value.toString()
        is Array<*> -> value.joinToString(prefix = "[", postfix = "]")
        else -> value.toString()
    }
}
