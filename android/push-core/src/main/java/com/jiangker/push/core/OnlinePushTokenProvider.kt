package com.jiangker.push.core

import android.content.Context
import java.util.UUID

internal object OnlinePushTokenProvider {
    private const val PREFS_NAME = "push_hub_online"
    private const val KEY_TOKEN = "push_token"

    fun getOrCreate(context: Context): String {
        val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val cached = prefs.getString(KEY_TOKEN, null)
        if (!cached.isNullOrBlank()) return cached

        val token = UUID.randomUUID().toString()
        prefs.edit().putString(KEY_TOKEN, token).apply()
        return token
    }
}
