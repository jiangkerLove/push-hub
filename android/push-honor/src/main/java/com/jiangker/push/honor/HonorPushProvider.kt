package com.jiangker.push.honor

import android.content.Context
import android.os.Build
import android.util.Log
import com.hihonor.push.sdk.HonorPushCallback
import com.hihonor.push.sdk.HonorPushClient
import com.jiangker.push.core.HonorConfig
import com.jiangker.push.core.ProcessUtils
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor

class HonorPushProvider : PushProvider {
    override val vendor: PushVendor = PushVendor.HONOR

    private lateinit var appContext: Context
    private var config: HonorConfig? = null

    override fun isSupported(context: Context): Boolean {
        if (!HonorPushClient.getInstance().checkSupportHonorPush(context)) return false
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return manufacturer.contains("honor") || brand.contains("honor")
    }

    override fun init(context: Context, config: PushConfig) {
        appContext = context.applicationContext
        this.config = config.honor
    }

    override fun register() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        if (config == null) return

        val client = HonorPushClient.getInstance()
        if (!client.checkSupportHonorPush(appContext)) {
            Log.w(TAG, "Honor push is not supported on this device")
            return
        }

        client.init(appContext, false)
        client.getPushToken(object : HonorPushCallback<String> {
            override fun onSuccess(token: String) {
                if (token.isNotBlank()) {
                    Log.i(TAG, "getPushToken success tokenLen=${token.length}")
                    PushEventDispatcher.dispatchToken(PushVendor.HONOR, token)
                } else {
                    Log.i(TAG, "getPushToken empty, waiting for onNewToken")
                }
            }

            override fun onFailure(errorCode: Int, errorMsg: String) {
                Log.w(
                    TAG,
                    "getPushToken failed code=$errorCode msg=$errorMsg, waiting for onNewToken",
                )
            }
        })
    }

    override fun unregister() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        HonorPushClient.getInstance().deletePushToken(object : HonorPushCallback<Void> {
            override fun onSuccess(result: Void?) {
                Log.i(TAG, "deletePushToken success")
            }

            override fun onFailure(errorCode: Int, errorMsg: String) {
                Log.w(TAG, "deletePushToken failed code=$errorCode msg=$errorMsg")
            }
        })
    }

    private companion object {
        const val TAG = "PushHub-Honor"
    }
}
