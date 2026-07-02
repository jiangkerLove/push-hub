package com.jiangker.push.core

internal data class OnlineMessageAck(
    val id: String,
    val displayed: Boolean,
    val reason: String? = null,
)
