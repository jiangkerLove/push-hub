package com.jiangker.push.core

import org.json.JSONObject

data class ClickAction(
    val type: String = "open_app",
    val activity: String? = null,
    val url: String? = null,
    val params: Map<String, Any?> = emptyMap(),
) {
    companion object {
        fun fromJson(json: JSONObject?): ClickAction? {
            if (json == null) return null
            val paramsJson = json.optJSONObject("params")
            val params = linkedMapOf<String, Any?>()
            if (paramsJson != null) {
                val keys = paramsJson.keys()
                while (keys.hasNext()) {
                    val key = keys.next()
                    params[key] = paramsJson.opt(key)
                }
            }
            return ClickAction(
                type = json.optString("type", "open_app").ifBlank { "open_app" },
                activity = json.optString("activity").takeIf { it.isNotBlank() },
                url = json.optString("url").takeIf { it.isNotBlank() },
                params = params,
            )
        }
    }
}
