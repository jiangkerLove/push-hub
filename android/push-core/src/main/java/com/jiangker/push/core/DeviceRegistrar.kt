package com.jiangker.push.core

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import org.json.JSONObject
import java.io.BufferedReader
import java.io.OutputStreamWriter
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.Executors
import androidx.core.content.edit

/**
 * 向 Push Hub 服务端注册设备，获取唯一 device id。
 *
 * 一台设备只有一个 [deviceId]：厂商 token 为 push_token，在线轮询 token 仅作 online_token。
 * 本地持久化该 id；厂商 pushId 刷新时带上旧 id，服务端只更新 token。
 */
object DeviceRegistrar {
    private const val TAG = "PushHub-Registrar"
    private const val PREFS_NAME = "push_hub_devices"
    private const val KEY_DEVICE_ID = "device_id"
    private const val KEY_VENDOR = "vendor"

    private lateinit var appContext: Context
    private var serverConfig: PushServerConfig? = null
    private var listener: PushRegistrationListener? = null
    private val executor = Executors.newSingleThreadExecutor()
    private val mainHandler = Handler(Looper.getMainLooper())

    @Volatile
    private var deviceId: String? = null

    @Volatile
    private var vendor: PushVendor? = null

    private data class PendingRegistration(val vendor: PushVendor, val pushToken: String)

    private val pendingRegistrations = mutableListOf<PendingRegistration>()

    fun init(
        context: Context,
        server: PushServerConfig,
        listener: PushRegistrationListener? = null,
    ) {
        appContext = context.applicationContext
        serverConfig = server
        this.listener = listener
        restore(server.hubAppId)
        flushPendingRegistrations()
    }

    fun setRegistrationListener(listener: PushRegistrationListener?) {
        this.listener = listener
    }

    /** 启动在线长连接（WebSocket），HTTP 轮询仅作断线兜底。 */
    fun startOnlinePolling() {
        val config = serverConfig ?: return
        if (!::appContext.isInitialized) return
        OnlinePushTokenProvider.getOrCreate(appContext)
        OnlinePushConnection.start(appContext, config)
    }

    fun register(vendor: PushVendor, pushToken: String) {
        val token = pushToken.trim()
        if (token.isEmpty()) return
        if (!::appContext.isInitialized || serverConfig == null) {
            Log.w(
                TAG,
                "register deferred: registrar not ready platform=${vendor.name.lowercase()}",
            )
            synchronized(pendingRegistrations) {
                pendingRegistrations.removeAll {
                    it.vendor == vendor && it.pushToken == token
                }
                pendingRegistrations += PendingRegistration(vendor, token)
            }
            return
        }

        val config = serverConfig ?: return
        val platform = vendor.platformName()
        val onlineToken = if (vendor != PushVendor.ONLINE) {
            OnlinePushTokenProvider.getOrCreate(appContext)
        } else {
            null
        }
        val boundDeviceId = deviceId

        executor.execute {
            runCatching {
                registerOnServer(config, platform, token, onlineToken, boundDeviceId)
            }.onSuccess { registeredId ->
                // 已有厂商身份时，纯 online 注册不覆盖主身份
                if (vendor == PushVendor.ONLINE && this.vendor != null && this.vendor != PushVendor.ONLINE) {
                    if (deviceId == null) {
                        deviceId = registeredId
                        persist(config.hubAppId, this.vendor ?: PushVendor.ONLINE, registeredId)
                    }
                } else {
                    deviceId = registeredId
                    this.vendor = vendor
                    persist(config.hubAppId, vendor, registeredId)
                }
                mainHandler.post {
                    listener?.onDeviceRegistered(platform, token, registeredId)
                    PushEventDispatcher.dispatchDeviceRegistered(vendor, registeredId, token)
                }
            }.onFailure { error ->
                val message = error.message ?: "device registration failed"
                Log.e(TAG, "register failed platform=$platform: $message", error)
                mainHandler.post {
                    listener?.onRegistrationFailed(platform, token, message)
                    PushEventDispatcher.dispatchError(
                        vendor,
                        IllegalStateException(message, error),
                    )
                }
            }
        }
    }

    /** 无厂商通道时的唯一设备注册（platform=online，push_token 即轮询 token）。 */
    fun registerOnlineOnlyDevice() {
        if (!::appContext.isInitialized) return
        val token = OnlinePushTokenProvider.getOrCreate(appContext)
        register(PushVendor.ONLINE, token)
    }

    fun getPrimaryDeviceId(): String? = deviceId

    fun getPrimaryVendor(): PushVendor = vendor ?: PushVendor.ONLINE

    fun getDeviceId(vendor: PushVendor): String? = deviceId

    fun getDeviceId(platform: String): String? = deviceId

    fun hasRegisteredDevice(): Boolean = deviceId != null

    private fun flushPendingRegistrations() {
        val pending = synchronized(pendingRegistrations) {
            pendingRegistrations.toList().also { pendingRegistrations.clear() }
        }
        pending.forEach { (vendor, token) ->
            register(vendor, token)
        }
    }

    private fun registerOnServer(
        config: PushServerConfig,
        platform: String,
        pushToken: String,
        onlineToken: String?,
        deviceId: String?,
    ): String {
        val url = URL("${config.baseUrl.trimEnd('/')}/api/v1/devices")
        val connection = (url.openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            connectTimeout = 15_000
            readTimeout = 15_000
            doOutput = true
            setRequestProperty("Content-Type", "application/json; charset=utf-8")
        }

        val body = JSONObject()
            .put("app_id", config.hubAppId)
            .put("package_name", config.packageName)
            .put("platform", platform)
            .put("push_token", pushToken)
        if (!deviceId.isNullOrBlank()) {
            body.put("device_id", deviceId)
        }
        if (!onlineToken.isNullOrBlank()) {
            body.put("online_token", onlineToken)
        }

        try {
            OutputStreamWriter(connection.outputStream, Charsets.UTF_8).use { writer ->
                writer.write(body.toString())
            }

            val status = connection.responseCode
            val responseText = readResponse(connection, status)
            if (status !in 200..299) {
                val error = runCatching {
                    JSONObject(responseText).optString("error", responseText)
                }.getOrDefault(responseText)
                throw IllegalStateException("HTTP $status: $error")
            }

            val json = JSONObject(responseText)
            val registeredId = json.optString("id").trim()
            if (registeredId.isEmpty()) {
                throw IllegalStateException("server response missing id")
            }
            if (!deviceId.isNullOrBlank() && registeredId == deviceId) {
                Log.i(TAG, "updated platform=$platform id=$registeredId pushId=$pushToken")
            } else {
                Log.i(TAG, "registered platform=$platform id=$registeredId pushId=$pushToken")
            }
            return registeredId
        } finally {
            connection.disconnect()
        }
    }

    private fun restore(hubAppId: String) {
        if (!::appContext.isInitialized) return
        val prefs = appContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val id = prefs.getString(scopedKey(KEY_DEVICE_ID, hubAppId), null)?.trim()
        val vendorName = prefs.getString(scopedKey(KEY_VENDOR, hubAppId), null)?.trim()
        if (!id.isNullOrBlank()) {
            deviceId = id
        }
        if (!vendorName.isNullOrBlank()) {
            vendor = runCatching {
                PushVendor.valueOf(vendorName.uppercase())
            }.getOrNull()
        }
        if (deviceId != null) {
            Log.i(TAG, "restored device_id=$deviceId vendor=${vendor?.name?.lowercase()}")
        }
    }

    private fun persist(hubAppId: String, vendor: PushVendor, deviceId: String) {
        if (!::appContext.isInitialized) return
        appContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit {
                putString(scopedKey(KEY_DEVICE_ID, hubAppId), deviceId)
                putString(scopedKey(KEY_VENDOR, hubAppId), vendor.name.lowercase())
            }
    }

    private fun scopedKey(base: String, hubAppId: String): String {
        val scope = hubAppId.trim().ifEmpty { "_" }
        return "${base}_$scope"
    }

    private fun readResponse(connection: HttpURLConnection, status: Int): String {
        val stream = if (status in 200..299) connection.inputStream else connection.errorStream
        if (stream == null) return ""
        return stream.bufferedReader().use(BufferedReader::readText)
    }
}

private fun PushVendor.platformName(): String = name.lowercase()
