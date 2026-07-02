package com.jiangker.push.vivo

import android.content.Context
import android.util.Log
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushVendor
import com.vivo.push.model.UPSNotificationMessage
import com.vivo.push.model.UnvarnishedMessage
import com.vivo.push.sdk.OpenClientPushMessageReceiver

class VivoPushMessageReceiver : OpenClientPushMessageReceiver() {
    override fun onReceiveRegId(context: Context?, regId: String?) {
        if (!regId.isNullOrBlank()) {
            Log.i(TAG, "onReceiveRegId pushId=$regId")
            PushEventDispatcher.dispatchToken(PushVendor.VIVO, regId)
        }
    }

    override fun onForegroundMessageArrived(message: UPSNotificationMessage?) {
        Log.i(TAG, "onForegroundMessageArrived message=$message")
        dispatchNotification(message, passThrough = false)
    }

    override fun onTransmissionMessage(context: Context?, message: UnvarnishedMessage?) {
        if (message == null) return
        Log.i(TAG, "onTransmissionMessage msgId=${message.msgId}")
        PushEventDispatcher.dispatchMessage(
            PushVendor.VIVO,
            PushMessage(
                title = null,
                content = message.message,
                payload = message.message,
                messageId = message.msgId.takeIf { it > 0 }?.toString(),
                passThrough = true,
            ),
        )
    }

    override fun onNotificationMessageClicked(context: Context?, message: UPSNotificationMessage?) {
        // 新版 SDK 通知点击由系统/Activity Intent 处理，此处仅作兼容日志
        Log.d(TAG, "onNotificationMessageClicked message=$message")
    }

    private fun dispatchNotification(message: UPSNotificationMessage?, passThrough: Boolean) {
        if (message == null) return
        val payload = message.customValue?.takeIf { it.isNotBlank() }
            ?: message.skipContent?.takeIf { it.isNotBlank() }
        PushEventDispatcher.dispatchMessage(
            PushVendor.VIVO,
            PushMessage(
                title = message.title,
                content = message.content,
                payload = payload,
                messageId = message.msgId.takeIf { it > 0 }?.toString(),
                passThrough = passThrough,
            ),
        )
    }

    private companion object {
        const val TAG = "PushHub-Vivo"
    }
}
