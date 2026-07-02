package com.jiangker.push.xiaomi

import android.content.Context
import android.util.Log
import com.jiangker.push.core.PushEventDispatcher
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushVendor
import com.xiaomi.mipush.sdk.ErrorCode
import com.xiaomi.mipush.sdk.MiPushClient
import com.xiaomi.mipush.sdk.MiPushCommandMessage
import com.xiaomi.mipush.sdk.MiPushMessage
import com.xiaomi.mipush.sdk.PushMessageReceiver

class XiaomiPushReceiver : PushMessageReceiver() {
    override fun onNotificationMessageClicked(context: Context, message: MiPushMessage) {
        // 服务端始终配置 notify_effect，点击由小米 SDK 直接打开目标页面，不会走到这里
        Log.d(TAG, "onNotificationMessageClicked ignored messageId=${message.messageId}")
    }

    override fun onReceivePassThroughMessage(context: Context, message: MiPushMessage) {
        Log.i(TAG, "onReceivePassThroughMessage messageId=${message.messageId}")
        PushEventDispatcher.dispatchMessage(
            PushVendor.XIAOMI,
            message.toPushMessage(passThrough = true),
        )
    }

    override fun onNotificationMessageArrived(context: Context, message: MiPushMessage) {
        Log.i(TAG, "onNotificationMessageArrived messageId=${message.messageId}")
        PushEventDispatcher.dispatchMessage(
            PushVendor.XIAOMI,
            message.toPushMessage(),
        )
    }

    override fun onCommandResult(context: Context, message: MiPushCommandMessage) {
        dispatchRegisterResult(message)
    }

    override fun onReceiveRegisterResult(context: Context, message: MiPushCommandMessage) {
        dispatchRegisterResult(message)
    }

    private fun dispatchRegisterResult(message: MiPushCommandMessage) {
        if (message.command != MiPushClient.COMMAND_REGISTER) return
        if (message.resultCode != ErrorCode.SUCCESS.toLong()) {
            PushEventDispatcher.dispatchError(
                PushVendor.XIAOMI,
                IllegalStateException("Xiaomi push register failed: ${message.reason}"),
            )
            return
        }

        val regId = message.commandArguments?.firstOrNull().orEmpty()
        if (regId.isNotEmpty()) {
            Log.i(TAG, "register success regId=$regId")
            PushEventDispatcher.dispatchToken(PushVendor.XIAOMI, regId)
        }
    }

    private companion object {
        const val TAG = "PushHub-Xiaomi"
    }
}

internal fun MiPushMessage.toPushMessage(passThrough: Boolean = false): PushMessage {
    val payloadText = content?.takeIf { it.isNotBlank() }
        ?: extra?.get("payload")?.toString()
    return PushMessage(
        title = title,
        content = description,
        payload = payloadText,
        extras = extra?.mapValues { it.value?.toString().orEmpty() }.orEmpty(),
        messageId = messageId,
        passThrough = passThrough,
    )
}
