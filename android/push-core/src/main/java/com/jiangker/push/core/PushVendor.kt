package com.jiangker.push.core

enum class PushVendor(val displayName: String) {
    XIAOMI("小米"),
    HUAWEI("华为"),
    HONOR("荣耀"),
    OPPO("OPPO"),
    VIVO("vivo"),
    MEIZU("魅族"),
    FCM("FCM"),
    /** 自建在线推送通道，适用于无厂商渠道包的场景 */
    ONLINE("在线推送"),
}
