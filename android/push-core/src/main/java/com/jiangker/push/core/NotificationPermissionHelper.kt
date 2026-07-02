package com.jiangker.push.core

import android.content.Context
import android.os.Build
import androidx.core.app.NotificationManagerCompat

internal enum class NotificationDisplayResult {
    DISPLAYED,
    PERMISSION_DENIED,
    NOT_APPLICABLE,
    ;

    fun toAckReason(): String? = when (this) {
        DISPLAYED -> null
        PERMISSION_DENIED -> "notification_permission_denied"
        NOT_APPLICABLE -> "data_message"
    }
}

internal object NotificationPermissionHelper {
    fun canShowNotifications(context: Context): Boolean {
        return NotificationManagerCompat.from(context).areNotificationsEnabled()
    }
}
