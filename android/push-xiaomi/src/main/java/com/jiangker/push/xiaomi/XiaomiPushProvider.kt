package com.jiangker.push.xiaomi

import android.content.Context
import android.os.Build
import android.util.Log
import com.jiangker.push.core.ProcessUtils
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor
import com.jiangker.push.core.XiaomiConfig
import com.xiaomi.mipush.sdk.MiPushClient

class XiaomiPushProvider : PushProvider {
    override val vendor: PushVendor = PushVendor.XIAOMI

    private lateinit var appContext: Context
    private var config: XiaomiConfig? = null

    override fun isSupported(context: Context): Boolean {
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return manufacturer.contains("xiaomi") ||
            manufacturer.contains("redmi") ||
            brand.contains("xiaomi") ||
            brand.contains("redmi")
    }

    override fun init(context: Context, config: PushConfig) {
        appContext = context.applicationContext
        this.config = config.xiaomi
    }

    override fun register() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        val xiaomiConfig = config ?: run {
            Log.w(TAG, "register skipped: xiaomi config missing")
            return
        }
        Log.i(TAG, "registerPush appId=${xiaomiConfig.appId}")
        MiPushClient.registerPush(appContext, xiaomiConfig.appId, xiaomiConfig.appKey)
    }

    override fun unregister() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        MiPushClient.unregisterPush(appContext)
    }

    private companion object {
        const val TAG = "PushHub-Xiaomi"
    }
}
