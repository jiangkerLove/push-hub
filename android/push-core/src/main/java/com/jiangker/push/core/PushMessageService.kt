package com.jiangker.push.core

import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.IBinder

/**
 * 推送消息接收 Service 基类。
 *
 * 通知点击由厂商 SDK 通过 [notify_effect] 直接打开应用 / Activity / 网页，
 * 无需业务侧实现点击回调。
 */
abstract class PushMessageService : Service() {

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent != null) {
            handleIntent(intent)
        }
        stopSelf(startId)
        return START_NOT_STICKY
    }

    internal fun deliverPushEvent(context: Context, intent: Intent) {
        attachBaseContext(context.applicationContext)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent) {
        val vendor = intent.getStringExtra(EXTRA_VENDOR)?.let { name ->
            runCatching { PushVendor.valueOf(name) }.getOrNull()
        } ?: return

        when (intent.action) {
            ACTION_TOKEN -> onNewToken(vendor, intent.getStringExtra(EXTRA_TOKEN).orEmpty())
            ACTION_DEVICE_REGISTERED -> onDeviceRegistered(
                vendor,
                intent.getStringExtra(EXTRA_DEVICE_ID).orEmpty(),
                intent.getStringExtra(EXTRA_TOKEN).orEmpty(),
            )
            ACTION_MESSAGE -> onMessageReceived(vendor, intent.toPushMessage())
            ACTION_ERROR -> onError(vendor, intent.getStringExtra(EXTRA_ERROR))
        }
    }

    open fun onNewToken(vendor: PushVendor, token: String) {}

    /** 服务端根据 platform + push_token 分配的设备 id */
    open fun onDeviceRegistered(vendor: PushVendor, deviceId: String, pushToken: String) {}

    open fun onMessageReceived(vendor: PushVendor, message: PushMessage) {}

    open fun onError(vendor: PushVendor, errorMessage: String?) {}

    companion object {
        const val ACTION_TOKEN = "com.jiangker.push.action.TOKEN"
        const val ACTION_DEVICE_REGISTERED = "com.jiangker.push.action.DEVICE_REGISTERED"
        const val ACTION_MESSAGE = "com.jiangker.push.action.MESSAGE"
        const val ACTION_ERROR = "com.jiangker.push.action.ERROR"

        const val EXTRA_VENDOR = "extra_vendor"
        const val EXTRA_TOKEN = "extra_token"
        const val EXTRA_DEVICE_ID = "extra_device_id"
        const val EXTRA_TITLE = "extra_title"
        const val EXTRA_CONTENT = "extra_content"
        const val EXTRA_PAYLOAD = "extra_payload"
        const val EXTRA_MESSAGE_ID = "extra_message_id"
        const val EXTRA_PASS_THROUGH = "extra_pass_through"
        const val EXTRA_EXTRAS = "extra_extras"
        const val EXTRA_ERROR = "extra_error"

        internal fun Intent.toPushMessage(): PushMessage {
            val extrasBundle = getBundleExtra(EXTRA_EXTRAS)
            val extras = extrasBundle?.keySet()
                ?.associateWith { key -> extrasBundle.getString(key).orEmpty() }
                .orEmpty()
            return PushMessage(
                title = getStringExtra(EXTRA_TITLE),
                content = getStringExtra(EXTRA_CONTENT),
                payload = getStringExtra(EXTRA_PAYLOAD),
                messageId = getStringExtra(EXTRA_MESSAGE_ID),
                passThrough = getBooleanExtra(EXTRA_PASS_THROUGH, false),
                extras = extras,
            )
        }
    }
}
