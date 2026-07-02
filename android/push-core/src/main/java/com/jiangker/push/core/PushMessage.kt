package com.jiangker.push.core

data class PushMessage(
    val title: String?,
    val content: String?,
    val payload: String? = null,
    val extras: Map<String, String> = emptyMap(),
    val messageId: String? = null,
    /** 通知栏 ID；相同 ID 的新消息覆盖旧通知。 */
    val notifyId: Int? = null,
    /** 是否为透传消息（不展示系统通知栏） */
    val passThrough: Boolean = false,
    val clickAction: ClickAction? = null,
)
