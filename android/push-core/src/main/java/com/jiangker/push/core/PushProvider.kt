package com.jiangker.push.core

import android.content.Context

interface PushProvider {
    val vendor: PushVendor

    fun isSupported(context: Context): Boolean

    fun init(context: Context, config: PushConfig)

    fun register()

    fun unregister()
}
