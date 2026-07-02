package com.jiangker.push.sample

import android.util.Log
import com.jiangker.push.core.PushMessage
import com.jiangker.push.core.PushMessageService
import com.jiangker.push.core.PushVendor

class SamplePushService : PushMessageService() {
    override fun onNewToken(vendor: PushVendor, token: String) {
        Log.i(TAG, "Push token [${vendor.displayName}]: $token")
    }

    override fun onDeviceRegistered(vendor: PushVendor, deviceId: String, pushToken: String) {
        Log.i(TAG, "Device registered [${vendor.displayName}]: id=$deviceId pushId=$pushToken")
    }

    override fun onMessageReceived(vendor: PushVendor, message: PushMessage) {
        Log.i(TAG, "Push message [${vendor.displayName}]: ${message.content}")
    }

    override fun onError(vendor: PushVendor, errorMessage: String?) {
        Log.e(TAG, "Push error [${vendor.displayName}]: $errorMessage")
    }

    private companion object {
        const val TAG = "SamplePushService"
    }
}
