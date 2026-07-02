package com.jiangker.push.meizu

import android.content.Context
import android.util.Log
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushVendor
import com.meizu.cloud.pushsdk.MzPushMessageReceiver
import com.meizu.cloud.pushsdk.handler.MzPushMessage
import com.meizu.cloud.pushsdk.platform.message.RegisterStatus

class MeizuPushMessageReceiver : MzPushMessageReceiver() {
    override fun onRegisterStatus(context: Context?, registerStatus: RegisterStatus?) {
        val pushId = registerStatus?.pushId
        if (!pushId.isNullOrBlank()) {
            Log.i(TAG, "onRegisterStatus pushId=$pushId")
            PushEventDispatcher.dispatchToken(PushVendor.MEIZU, pushId)
        } else {
            Log.w(TAG, "onRegisterStatus without pushId code=${registerStatus?.code}")
            PushEventDispatcher.dispatchError(
                PushVendor.MEIZU,
                IllegalStateException(
                    "Meizu push register failed: code=${registerStatus?.code} msg=${registerStatus?.message}",
                ),
            )
        }
    }

    override fun onNotificationArrived(context: Context?, message: MzPushMessage?) {
        Log.d(TAG, "onNotificationArrived message=$message")
        if (message == null) return
        PushEventDispatcher.dispatchMessage(
            PushVendor.MEIZU,
            PushMessage(
                title = message.title,
                content = message.content,
                payload = message.selfDefineContentString,
                messageId = message.notifyId.takeIf { it > 0 }?.toString(),
                passThrough = false,
            ),
        )
    }

    override fun onPushStatus(context: Context?, pushSwitchStatus: com.meizu.cloud.pushsdk.platform.message.PushSwitchStatus?) {
        Log.d(TAG, "onPushStatus pushSwitchStatus=$pushSwitchStatus")
    }

    private companion object {
        const val TAG = "PushHub-Meizu"
    }
}
