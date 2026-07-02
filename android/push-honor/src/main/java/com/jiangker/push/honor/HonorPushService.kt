package com.jiangker.push.honor

import android.util.Log
import com.hihonor.push.sdk.HonorMessageService
import com.hihonor.push.sdk.HonorPushDataMsg
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushVendor

class HonorPushService : HonorMessageService() {
    override fun onNewToken(token: String) {
        Log.i(TAG, "onNewToken tokenLen=${token.length}")
        if (token.isNotBlank()) {
            PushEventDispatcher.dispatchToken(PushVendor.HONOR, token)
        }
    }

    override fun onMessageReceived(dataMessage: HonorPushDataMsg) {
        Log.i(TAG, "onMessageReceived msgId=${dataMessage.msgId}")
        val payload = dataMessage.data
        PushEventDispatcher.dispatchMessage(
            PushVendor.HONOR,
            PushMessage(
                title = null,
                content = payload,
                payload = payload,
                messageId = dataMessage.msgId.takeIf { it > 0 }?.toString(),
                passThrough = true,
            ),
        )
    }

    private companion object {
        const val TAG = "PushHub-Honor"
    }
}
