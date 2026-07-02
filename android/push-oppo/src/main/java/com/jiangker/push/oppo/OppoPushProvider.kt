package com.jiangker.push.oppo

import android.content.Context
import android.os.Build
import android.util.Log
import com.heytap.msp.push.HeytapPushManager
import com.heytap.msp.push.callback.ICallBackResultService
import com.heytap.msp.push.mode.ErrorCode
import com.jiangker.push.core.OppoConfig
import com.jiangker.push.core.ProcessUtils
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor

class OppoPushProvider : PushProvider {
    override val vendor: PushVendor = PushVendor.OPPO

    private lateinit var appContext: Context
    private var config: OppoConfig? = null

    override fun isSupported(context: Context): Boolean {
        if (!HeytapPushManager.isSupportPush(context)) return false
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return manufacturer.contains("oppo") ||
            manufacturer.contains("realme") ||
            manufacturer.contains("oneplus") ||
            brand.contains("oppo") ||
            brand.contains("realme") ||
            brand.contains("oneplus")
    }

    override fun init(context: Context, config: PushConfig) {
        appContext = context.applicationContext
        this.config = config.oppo
    }

    override fun register() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        val oppoConfig = config ?: return

        HeytapPushManager.init(appContext, true)
        if (!HeytapPushManager.isSupportPush(appContext)) {
            Log.w(TAG, "OPPO push is not supported on this device")
            return
        }

        HeytapPushManager.register(
            appContext,
            oppoConfig.appKey,
            oppoConfig.appSecret,
            pushCallback,
        )
        HeytapPushManager.requestNotificationPermission()
    }

    override fun unregister() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        HeytapPushManager.unRegister()
    }

    private val pushCallback = object : ICallBackResultService {
        override fun onRegister(
            code: Int,
            registerId: String?,
            packageName: String?,
            miniPackageName: String?,
        ) {
            if (code == ErrorCode.SUCCESS && !registerId.isNullOrBlank()) {
                Log.i(TAG, "register success registerId=$registerId")
                PushEventDispatcher.dispatchToken(PushVendor.OPPO, registerId)
                return
            }
            Log.w(
                TAG,
                "register failed code=$code registerId=$registerId pkg=$packageName mini=$miniPackageName",
            )
            PushEventDispatcher.dispatchError(
                PushVendor.OPPO,
                IllegalStateException("OPPO push register failed: code=$code"),
            )
        }

        override fun onUnRegister(code: Int, packageName: String?, miniPackageName: String?) {
            Log.i(TAG, "onUnRegister code=$code")
        }

        override fun onSetPushTime(code: Int, pushTime: String?) {
            Log.d(TAG, "onSetPushTime code=$code pushTime=$pushTime")
        }

        override fun onGetPushStatus(code: Int, status: Int) {
            Log.d(TAG, "onGetPushStatus code=$code status=$status")
        }

        override fun onGetNotificationStatus(code: Int, status: Int) {
            Log.d(TAG, "onGetNotificationStatus code=$code status=$status")
        }

        override fun onError(
            errorCode: Int,
            message: String?,
            packageName: String?,
            miniPackageName: String?,
        ) {
            Log.w(TAG, "onError code=$errorCode message=$message")
            PushEventDispatcher.dispatchError(
                PushVendor.OPPO,
                IllegalStateException("OPPO push error: code=$errorCode message=$message"),
            )
        }
    }

    private companion object {
        const val TAG = "PushHub-Oppo"
    }
}
