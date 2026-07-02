package com.jiangker.push.meizu

import android.content.Context
import android.os.Build
import android.util.Log
import com.jiangker.push.core.MeizuConfig
import com.jiangker.push.core.ProcessUtils
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor
import com.meizu.cloud.pushsdk.PushManager

class MeizuPushProvider : PushProvider {
    override val vendor: PushVendor = PushVendor.MEIZU

    private lateinit var appContext: Context
    private var config: MeizuConfig? = null

    override fun isSupported(context: Context): Boolean {
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return manufacturer.contains("meizu") || brand.contains("meizu")
    }

    override fun init(context: Context, config: PushConfig) {
        appContext = context.applicationContext
        this.config = config.meizu
    }

    override fun register() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        val meizuConfig = config ?: return

        PushManager.register(appContext, meizuConfig.appId, meizuConfig.appKey)
        Log.i(TAG, "PushManager.register called appId=${meizuConfig.appId}")
    }

    override fun unregister() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        val meizuConfig = config ?: return
        PushManager.unRegister(appContext, meizuConfig.appId, meizuConfig.appKey)
    }

    private companion object {
        const val TAG = "PushHub-Meizu"
    }
}
