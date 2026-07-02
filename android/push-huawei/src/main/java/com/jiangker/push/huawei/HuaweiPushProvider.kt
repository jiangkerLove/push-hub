package com.jiangker.push.huawei

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.util.Log
import com.huawei.hms.aaid.HmsInstanceId
import com.huawei.hms.common.ApiException
import com.huawei.hms.push.HmsMessaging
import com.jiangker.push.core.ProcessUtils
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor
import java.util.concurrent.Executors

/**
 * 华为厂商通道适配器。
 *
 * App ID 由 Manifest meta-data（manifestPlaceholders）自动注入，或通过
 * [com.jiangker.push.PushHubConfig.Builder.huawei] 显式覆盖。
 */
class HuaweiPushProvider : PushProvider {
    override val vendor: PushVendor = PushVendor.HUAWEI

    private lateinit var appContext: Context
    private var enabled = false
    private var appId: String? = null
    private val executor = Executors.newSingleThreadExecutor()
    private val mainHandler = Handler(Looper.getMainLooper())

    override fun isSupported(context: Context): Boolean {
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return manufacturer.contains("huawei") || brand.contains("huawei")
    }

    override fun init(context: Context, config: PushConfig) {
        appContext = context.applicationContext
        val huawei = config.huawei
        if (huawei == null) {
            enabled = false
            return
        }

        appId = huawei.appId.takeIf { it.isNotBlank() }
            ?: resolveAppIdFromManifest(appContext)
        if (appId.isNullOrBlank()) {
            Log.e(TAG, "Huawei app id missing: configure HUAWEI_APP_ID in manifestPlaceholders")
            enabled = false
            return
        }
        enabled = true

        runCatching {
            HmsMessaging.getInstance(appContext).isAutoInitEnabled = false
        }
    }

    override fun register() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        if (!enabled) return

        runCatching {
            HmsMessaging.getInstance(appContext).isAutoInitEnabled = true
        }.onFailure { error ->
            Log.e(TAG, "enable Huawei push failed: ${error.message}", error)
        }

        requestToken("register", attempt = 1)
    }

    private fun requestToken(source: String, attempt: Int) {
        val resolvedAppId = appId ?: return
        executor.execute {
            runCatching {
                HmsInstanceId.getInstance(appContext).getToken(resolvedAppId, "HCM")
            }.onSuccess { token ->
                if (token.isNullOrBlank()) {
                    Log.w(TAG, "getToken($source) returned empty, waiting for onNewToken")
                    scheduleRetry(source, attempt)
                    return@onSuccess
                }
                Log.i(TAG, "getToken($source) pushId=$token")
                PushEventDispatcher.dispatchToken(PushVendor.HUAWEI, token)
            }.onFailure { error ->
                val detail = when (error) {
                    is ApiException -> "code=${error.statusCode} msg=${error.message}"
                    else -> error.message
                }
                Log.w(
                    TAG,
                    "getToken($source) failed: $detail, waiting for onNewToken",
                    error,
                )
                scheduleRetry(source, attempt)
            }
        }
    }

    private fun scheduleRetry(source: String, attempt: Int) {
        if (attempt >= MAX_TOKEN_ATTEMPTS) return
        mainHandler.postDelayed(
            { requestToken(source, attempt + 1) },
            TOKEN_RETRY_DELAY_MS,
        )
    }

    override fun unregister() {
        if (!ProcessUtils.isMainProcess(appContext)) return
        if (!enabled) return
        val resolvedAppId = appId ?: return
        executor.execute {
            runCatching {
                HmsInstanceId.getInstance(appContext).deleteToken(resolvedAppId, "HCM")
            }.onFailure { error ->
                Log.w(TAG, "deleteToken failed: ${error.message}")
            }
        }
    }

    private companion object {
        const val TAG = "PushHub-Huawei"
        const val MAX_TOKEN_ATTEMPTS = 3
        const val TOKEN_RETRY_DELAY_MS = 2_000L

        fun resolveAppIdFromManifest(context: Context): String? {
            val raw = runCatching {
                val appInfo = context.packageManager.getApplicationInfo(
                    context.packageName,
                    PackageManager.GET_META_DATA,
                )
                appInfo.metaData?.getString("com.huawei.hms.client.appid")
            }.getOrNull() ?: return null
            val trimmed = raw.trim()
            if (trimmed.isEmpty()) return null
            return trimmed.removePrefix("appid=").trim().ifBlank { trimmed }
        }
    }
}
