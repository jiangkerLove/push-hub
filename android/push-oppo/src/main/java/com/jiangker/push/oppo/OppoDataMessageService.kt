package com.jiangker.push.oppo

import android.content.Context
import android.util.Log
import com.heytap.msp.push.mode.DataMessage
import com.heytap.msp.push.service.CompatibleDataMessageCallbackService
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushVendor

class OppoDataMessageService : CompatibleDataMessageCallbackService() {
    override fun processMessage(context: Context, message: DataMessage) {
        Log.i(TAG, "processMessage messageId=${message.messageID}")
        PushEventDispatcher.dispatchMessage(
            PushVendor.OPPO,
            PushMessage(
                title = message.title,
                content = message.content ?: message.description,
                payload = message.dataExtra ?: message.content,
                messageId = message.messageID,
                passThrough = true,
            ),
        )
    }

    private companion object {
        const val TAG = "PushHub-Oppo"
    }
}
