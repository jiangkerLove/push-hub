package com.jiangker.push.core

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.json.JSONArray
import org.json.JSONObject
import java.net.URLEncoder
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

/**
 * 在线推送 WebSocket 长连接：服务端 outbox 写入后实时下发。
 * 断线时自动重连，并用 HTTP 轮询兜底未送达消息。
 */
internal object OnlinePushConnection {
    private const val TAG = "PushHub-Online"
    private const val RECONNECT_MS = 5_000L
    private const val HEARTBEAT_MS = 30_000L

    private val mainHandler = Handler(Looper.getMainLooper())
    private val running = AtomicBoolean(false)
    private val client = OkHttpClient.Builder()
        .pingInterval(30, TimeUnit.SECONDS)
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(0, TimeUnit.SECONDS)
        .build()

    private lateinit var appContext: Context
    private var serverConfig: PushServerConfig? = null
    private var webSocket: WebSocket? = null

    fun start(context: Context, server: PushServerConfig) {
        appContext = context.applicationContext
        serverConfig = server
        if (!running.compareAndSet(false, true)) return
        Log.i(TAG, "online websocket starting")
        connect()
        scheduleHeartbeat()
    }

    fun stop() {
        running.set(false)
        mainHandler.removeCallbacksAndMessages(null)
        webSocket?.close(1000, "stopped")
        webSocket = null
    }

    private fun connect() {
        if (!running.get()) return
        val server = serverConfig ?: return
        val pushToken = OnlinePushTokenProvider.getOrCreate(appContext)
        val wsBase = server.baseUrl.trimEnd('/')
            .replace("https://", "wss://")
            .replace("http://", "ws://")
        val encodedToken = URLEncoder.encode(pushToken, Charsets.UTF_8.name())
        val url = "$wsBase/api/v1/online/ws?push_token=$encodedToken"

        webSocket?.cancel()
        val request = Request.Builder().url(url).build()
        webSocket = client.newWebSocket(request, socketListener)
    }

    private val socketListener = object : WebSocketListener() {
        override fun onOpen(webSocket: WebSocket, response: Response) {
            Log.i(TAG, "websocket connected")
        }

        override fun onMessage(webSocket: WebSocket, text: String) {
            handleServerText(text)
        }

        override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
            webSocket.close(code, reason)
        }

        override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
            Log.i(TAG, "websocket closed code=$code reason=$reason")
            scheduleReconnect()
        }

        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
            Log.w(TAG, "websocket failure: ${t.message}")
            scheduleReconnect()
            pollFallbackOnce()
        }
    }

    private fun handleServerText(text: String) {
        runCatching {
            val json = JSONObject(text)
            when (json.optString("type")) {
                "message" -> {
                    val message = OnlineOutboxMessage(
                        id = json.optString("id"),
                        title = json.optString("title"),
                        body = json.optString("body"),
                        payload = json.opt("payload")?.toString().orEmpty(),
                        deliveryMode = json.optString("delivery_mode", "notification"),
                        notifyId = if (json.has("notify_id") && !json.isNull("notify_id")) {
                            json.getInt("notify_id")
                        } else {
                            null
                        },
                        clickAction = ClickAction.fromJson(json.optJSONObject("click_action")),
                    )
                    if (message.id.isNotBlank()) {
                        deliverMessages(listOf(message))
                    }
                }
                "pong" -> Unit
            }
        }.onFailure { error ->
            Log.w(TAG, "invalid websocket payload: ${error.message}")
        }
    }

    private fun deliverMessages(messages: List<OnlineOutboxMessage>) {
        if (messages.isEmpty()) return
        val vendor = DeviceRegistrar.getPrimaryVendor()

        mainHandler.post {
            val acks = messages.mapNotNull { message ->
                val passThrough = message.deliveryMode == "data"
                val payload = message.payload.ifBlank { null }
                val pushMessage = PushMessage(
                    title = message.title.takeIf { it.isNotBlank() },
                    content = message.body.takeIf { it.isNotBlank() },
                    payload = payload,
                    messageId = message.id,
                    notifyId = message.notifyId,
                    passThrough = passThrough,
                    clickAction = message.clickAction,
                )

                val displayResult = if (passThrough) {
                    NotificationDisplayResult.NOT_APPLICABLE
                } else {
                    OnlinePushNotifier.showNotification(appContext, pushMessage)
                }
                PushEventDispatcher.dispatchMessage(vendor, pushMessage)
                buildAck(message.id, displayResult)
            }
            ackDelivered(acks)
            Log.i(TAG, "delivered ${acks.size} message(s) via online channel")
        }
    }

    private fun buildAck(id: String, displayResult: NotificationDisplayResult): OnlineMessageAck {
        val reason = displayResult.toAckReason()
        return OnlineMessageAck(
            id = id,
            displayed = displayResult == NotificationDisplayResult.DISPLAYED,
            reason = reason,
        )
    }

    private fun ackDelivered(acks: List<OnlineMessageAck>) {
        if (acks.isEmpty()) return
        val ws = webSocket
        if (ws != null) {
            val ackArray = JSONArray()
            acks.forEach { ack ->
                ackArray.put(
                    JSONObject()
                        .put("id", ack.id)
                        .put("displayed", ack.displayed)
                        .also { json ->
                            ack.reason?.let { json.put("reason", it) }
                        },
                )
            }
            val body = JSONObject()
                .put("type", "ack")
                .put("acks", ackArray)
                .toString()
            if (ws.send(body)) return
        }

        val server = serverConfig ?: return
        val pushToken = OnlinePushTokenProvider.getOrCreate(appContext)
        runCatching {
            OnlinePushClient.ack(server, pushToken, acks)
        }.onFailure { error ->
            Log.w(TAG, "http ack failed: ${error.message}")
        }
    }

    private fun scheduleReconnect() {
        if (!running.get()) return
        mainHandler.removeCallbacks(reconnectRunnable)
        mainHandler.postDelayed(reconnectRunnable, RECONNECT_MS)
    }

    private val reconnectRunnable = Runnable { connect() }

    private fun scheduleHeartbeat() {
        if (!running.get()) return
        mainHandler.postDelayed(heartbeatRunnable, HEARTBEAT_MS)
    }

    private val heartbeatRunnable = object : Runnable {
        override fun run() {
            if (!running.get()) return
            webSocket?.send("""{"type":"ping"}""")
            mainHandler.postDelayed(this, HEARTBEAT_MS)
        }
    }

    private fun pollFallbackOnce() {
        val server = serverConfig ?: return
        val pushToken = OnlinePushTokenProvider.getOrCreate(appContext)
        Thread {
            runCatching {
                val messages = OnlinePushClient.fetchPending(server, pushToken)
                if (messages.isNotEmpty()) {
                    deliverMessages(messages)
                }
            }.onFailure { error ->
                Log.w(TAG, "fallback poll failed: ${error.message}")
            }
        }.start()
    }
}
