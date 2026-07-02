package com.jiangker.push.core

/**
 * 设备注册结果回调，应用层可直接实现此接口获取服务端分配的 id。
 */
fun interface PushRegistrationListener {
    fun onDeviceRegistered(platform: String, pushToken: String, deviceId: String)

    fun onRegistrationFailed(platform: String, pushToken: String, error: String) {}
}
