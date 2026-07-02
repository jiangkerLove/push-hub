package com.jiangker.push.vivo

import android.content.Context
import android.os.Build
import android.util.Log
import com.jiangker.push.core.ProcessUtils
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor
import com.jiangker.push.core.VivoConfig
import com.vivo.push.IPushActionListener
import com.vivo.push.PushClient
import com.vivo.push.listener.IPushQueryActionListener
import com.vivo.push.util.VivoPushException

class VivoPushProvider : PushProvider {
    override val vendor: PushVendor = PushVendor.VIVO

    private lateinit var appContext: Context
    private var config: VivoConfig? = null

    override fun isSupported(context: Context): Boolean {
        if (!PushClient.getInstance(context).isSupport) return false
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return manufacturer.contains("vivo") ||
            manufacturer.contains("iqoo") ||
            brand.contains("vivo") ||
            brand.contains("iqoo")
    }

    override fun init(context: Context, config: PushConfig) {
        appContext = context.applicationContext
        this.config = config.vivo
    }

    override fun register() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        val vivoConfig = config ?: run {
            Log.w(TAG, "register skipped: vivo config missing, call PushHubConfig.vivo()")
            return
        }
        Log.i(TAG, "register vivo appId=${vivoConfig.appId}")

        val client = PushClient.getInstance(appContext)
        try {
            val sdkConfig = com.vivo.push.PushConfig.Builder()
                .agreePrivacyStatement(true)
                .build()
            client.initialize(sdkConfig)
        } catch (error: VivoPushException) {
            Log.e(TAG, "initialize failed: ${error.message}", error)
            PushEventDispatcher.dispatchError(
                PushVendor.VIVO,
                IllegalStateException("Vivo push initialize failed: ${error.message}", error),
            )
            return
        }

        if (!client.isSupport) {
            Log.w(TAG, "Vivo push is not supported on this device")
            return
        }

        client.turnOnPush(object : IPushActionListener {
            override fun onStateChanged(state: Int) {
                if (state == 0) {
                    Log.i(TAG, "turnOnPush success, querying regId")
                    queryRegId(client)
                } else {
                    Log.w(TAG, "turnOnPush failed state=$state")
                    PushEventDispatcher.dispatchError(
                        PushVendor.VIVO,
                        IllegalStateException("Vivo push turnOnPush failed: state=$state"),
                    )
                }
            }
        })
    }

    override fun unregister() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        PushClient.getInstance(appContext).turnOffPush(object : IPushActionListener {
            override fun onStateChanged(state: Int) {
                Log.i(TAG, "turnOffPush state=$state")
            }
        })
    }

    private fun queryRegId(client: PushClient) {
        client.getRegId(object : IPushQueryActionListener {
            override fun onSuccess(regId: String?) {
                if (!regId.isNullOrBlank()) {
                    Log.i(TAG, "getRegId success pushId=$regId")
                    PushEventDispatcher.dispatchToken(PushVendor.VIVO, regId)
                } else {
                    Log.i(TAG, "getRegId empty, waiting for onReceiveRegId")
                }
            }

            override fun onFail(errorCode: Int?) {
                Log.w(TAG, "getRegId failed code=$errorCode, waiting for onReceiveRegId")
            }
        })
    }

    private companion object {
        const val TAG = "PushHub-Vivo"
    }
}
