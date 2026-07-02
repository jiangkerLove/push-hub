package com.jiangker.push.core

import org.json.JSONArray
import org.json.JSONObject
import java.io.BufferedReader
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

internal data class OnlineOutboxMessage(
    val id: String,
    val title: String,
    val body: String,
    val payload: String,
    val deliveryMode: String,
    val notifyId: Int? = null,
    val clickAction: ClickAction? = null,
)

internal object OnlinePushClient {
    fun fetchPending(server: PushServerConfig, pushToken: String): List<OnlineOutboxMessage> {
        val encodedToken = URLEncoder.encode(pushToken, Charsets.UTF_8.name())
        val url = URL(
            "${server.baseUrl.trimEnd('/')}/api/v1/online/messages?push_token=$encodedToken&limit=20",
        )
        val connection = (url.openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 15_000
            readTimeout = 15_000
        }

        try {
            val status = connection.responseCode
            val responseText = readResponse(connection, status)
            if (status !in 200..299) {
                throw IllegalStateException("HTTP $status: $responseText")
            }
            return parseMessages(responseText)
        } finally {
            connection.disconnect()
        }
    }

    fun ack(server: PushServerConfig, pushToken: String, acks: List<OnlineMessageAck>) {
        if (acks.isEmpty()) return

        val url = URL("${server.baseUrl.trimEnd('/')}/api/v1/online/messages")
        val connection = (url.openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            connectTimeout = 15_000
            readTimeout = 15_000
            doOutput = true
            setRequestProperty("Content-Type", "application/json; charset=utf-8")
        }

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
            .put("push_token", pushToken)
            .put("acks", ackArray)
            .toString()

        try {
            connection.outputStream.bufferedWriter(Charsets.UTF_8).use { it.write(body) }
            val status = connection.responseCode
            if (status !in 200..299) {
                val responseText = readResponse(connection, status)
                throw IllegalStateException("HTTP $status: $responseText")
            }
        } finally {
            connection.disconnect()
        }
    }

    private fun parseMessages(responseText: String): List<OnlineOutboxMessage> {
        val array = JSONArray(responseText)
        return buildList {
            for (index in 0 until array.length()) {
                val item = array.getJSONObject(index)
                add(
                    OnlineOutboxMessage(
                        id = item.optString("id"),
                        title = item.optString("title"),
                        body = item.optString("body"),
                        payload = item.opt("payload")?.toString().orEmpty(),
                        deliveryMode = item.optString("delivery_mode", "notification"),
                        notifyId = if (item.has("notify_id") && !item.isNull("notify_id")) {
                            item.getInt("notify_id")
                        } else {
                            null
                        },
                        clickAction = ClickAction.fromJson(item.optJSONObject("click_action")),
                    ),
                )
            }
        }.filter { it.id.isNotBlank() }
    }

    private fun readResponse(connection: HttpURLConnection, status: Int): String {
        val stream = if (status in 200..299) connection.inputStream else connection.errorStream
        if (stream == null) return ""
        return stream.bufferedReader().use(BufferedReader::readText)
    }
}
