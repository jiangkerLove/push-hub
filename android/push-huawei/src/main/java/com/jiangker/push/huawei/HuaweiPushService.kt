package com.jiangker.push.huawei

import android.os.Bundle
import android.util.Log
import com.huawei.hms.push.HmsMessageService
import com.huawei.hms.push.RemoteMessage
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushVendor

class HuaweiPushService : HmsMessageService() {
    override fun onNewToken(token: String) {
        super.onNewToken(token)
        dispatchToken(token)
    }

    override fun onNewToken(token: String, bundle: Bundle) {
        super.onNewToken(token, bundle)
        dispatchToken(token)
    }

    private fun dispatchToken(token: String) {
        Log.i(TAG, "onNewToken pushId=$token")
        if (token.isNotBlank()) {
            PushEventDispatcher.dispatchToken(PushVendor.HUAWEI, token)
        }
    }

    override fun onMessageReceived(message: RemoteMessage) {
        Log.i(TAG, "onMessageReceived messageId=${message.messageId}")
        val isDataOnly = message.notification == null
        PushEventDispatcher.dispatchMessage(
            PushVendor.HUAWEI,
            PushMessage(
                title = message.notification?.title,
                content = message.notification?.body,
                payload = message.data,
                messageId = message.messageId,
                passThrough = isDataOnly,
            ),
        )
    }

    private companion object {
        const val TAG = "PushHub-Huawei"
    }
}
