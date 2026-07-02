package com.jiangker.push.core

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import org.json.JSONObject

internal object OnlinePushNotifier {
    private const val CHANNEL_ID = "push_hub_online"
    private const val CHANNEL_NAME = "在线推送"

    fun showNotification(context: Context, message: PushMessage): NotificationDisplayResult {
        ensureChannel(context)

        val title = message.title?.takeIf { it.isNotBlank() } ?: "新消息"
        val content = message.content?.takeIf { it.isNotBlank() } ?: message.payload.orEmpty()
        val notificationId = resolveNotificationId(message, content)

        if (!NotificationPermissionHelper.canShowNotifications(context)) {
            return NotificationDisplayResult.PERMISSION_DENIED
        }

        val contentIntent = buildContentIntent(context, message, notificationId)
        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(title)
            .setContentText(content)
            .setStyle(NotificationCompat.BigTextStyle().bigText(content))
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(contentIntent)
            .build()

        return try {
            NotificationManagerCompat.from(context).notify(notificationId, notification)
            NotificationDisplayResult.DISPLAYED
        } catch (_: SecurityException) {
            NotificationDisplayResult.PERMISSION_DENIED
        }
    }

    private fun resolveNotificationId(message: PushMessage, content: String): Int {
        message.notifyId?.let { return it and 0x7FFFFFFF }
        val raw = message.messageId?.hashCode() ?: content.hashCode()
        val id = raw and 0x7FFFFFFF
        return if (id == 0) 1 else id
    }

    private fun buildContentIntent(
        context: Context,
        message: PushMessage,
        notificationId: Int,
    ): PendingIntent {
        val action = message.clickAction ?: ClickAction()
        val intent = when (action.type) {
            "open_page" -> buildOpenPageIntent(context, action)
            "open_web" -> buildOpenWebIntent(action)
            else -> buildOpenAppIntent(context)
        }.apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        }

        val flags = PendingIntent.FLAG_UPDATE_CURRENT or
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0
        val requestCode = message.notifyId ?: notificationId
        return PendingIntent.getActivity(context, requestCode, intent, flags)
    }

    private fun buildOpenAppIntent(context: Context): Intent {
        return context.packageManager.getLaunchIntentForPackage(context.packageName)
            ?: Intent().setPackage(context.packageName).addCategory(Intent.CATEGORY_LAUNCHER)
    }

    private fun buildOpenPageIntent(context: Context, action: ClickAction): Intent {
        val activity = action.activity?.trim().orEmpty()
        if (activity.isBlank()) {
            return buildOpenAppIntent(context)
        }
        return Intent().apply {
            setClassName(context.packageName, activity)
            putClickParams(action.params)
        }
    }

    private fun buildOpenWebIntent(action: ClickAction): Intent {
        val url = action.url?.trim().orEmpty()
        if (url.isBlank()) {
            return Intent(Intent.ACTION_MAIN)
        }
        return Intent(Intent.ACTION_VIEW, Uri.parse(url))
    }

    private fun Intent.putClickParams(params: Map<String, Any?>) {
        params.forEach { (key, value) ->
            when (value) {
                null, JSONObject.NULL -> Unit
                is Boolean -> putExtra(key, value)
                is Int -> putExtra(key, value)
                is Long -> putExtra(key, value)
                is Double -> putExtra(key, value)
                is Float -> putExtra(key, value)
                is Number -> {
                    val asLong = value.toLong()
                    if (asLong.toDouble() == value.toDouble() &&
                        asLong in Int.MIN_VALUE.toLong()..Int.MAX_VALUE.toLong()
                    ) {
                        putExtra(key, asLong.toInt())
                    } else if (asLong.toDouble() == value.toDouble()) {
                        putExtra(key, asLong)
                    } else {
                        putExtra(key, value.toDouble())
                    }
                }
                else -> putExtra(key, value.toString())
            }
        }
    }

    private fun ensureChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        manager.createNotificationChannel(
            NotificationChannel(
                CHANNEL_ID,
                CHANNEL_NAME,
                NotificationManager.IMPORTANCE_DEFAULT,
            ),
        )
    }
}
