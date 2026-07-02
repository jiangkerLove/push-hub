package com.jiangker.push.sample

import android.app.Application
import android.util.Log
import com.jiangker.push.PushHub
import com.jiangker.push.PushHubConfig

class SampleApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "Application onCreate pid=${android.os.Process.myPid()}")
        PushHub.init(
            context = this,
            config = PushHubConfig.Builder()
                .messageService(SamplePushService::class.java)
                .registrationListener { platform, pushToken, deviceId ->
                    Log.i(TAG, "App got device id [$platform]: id=$deviceId pushId=$pushToken")
                }
                .build(this),
        )
    }

    private companion object {
        const val TAG = "SampleApplication"
    }
}
