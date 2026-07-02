package com.jiangker.push.core

import android.content.Context
import android.content.Intent
import android.os.Handler
import android.os.Looper

object PushEventDispatcher {
    private lateinit var appContext: Context
    private lateinit var serviceClass: Class<out PushMessageService>
    private val mainHandler = Handler(Looper.getMainLooper())

    fun init(context: Context, serviceClass: Class<out PushMessageService>) {
        appContext = context.applicationContext
        this.serviceClass = serviceClass
    }

    fun isInitialized(): Boolean = this::appContext.isInitialized && this::serviceClass.isInitialized

    fun dispatchToken(vendor: PushVendor, token: String) {
        DeviceRegistrar.register(vendor, token)
        dispatch(
            action = PushMessageService.ACTION_TOKEN,
            vendor = vendor,
        ) {
            putExtra(PushMessageService.EXTRA_TOKEN, token)
        }
    }

    fun dispatchDeviceRegistered(vendor: PushVendor, deviceId: String, pushToken: String) {
        dispatch(
            action = PushMessageService.ACTION_DEVICE_REGISTERED,
            vendor = vendor,
        ) {
            putExtra(PushMessageService.EXTRA_DEVICE_ID, deviceId)
            putExtra(PushMessageService.EXTRA_TOKEN, pushToken)
        }
    }

    fun dispatchMessage(vendor: PushVendor, message: PushMessage) {
        dispatch(
            action = PushMessageService.ACTION_MESSAGE,
            vendor = vendor,
        ) {
            putPushMessage(message)
        }
    }

    fun dispatchError(vendor: PushVendor, error: Throwable) {
        dispatch(
            action = PushMessageService.ACTION_ERROR,
            vendor = vendor,
        ) {
            putExtra(PushMessageService.EXTRA_ERROR, error.message)
        }
    }

    private inline fun dispatch(
        action: String,
        vendor: PushVendor,
        block: Intent.() -> Unit,
    ) {
        if (!isInitialized()) return

        val intent = Intent(appContext, serviceClass).apply {
            this.action = action
            putExtra(PushMessageService.EXTRA_VENDOR, vendor.name)
            block()
        }
        mainHandler.post {
            PushMessageServiceDeliverer.deliver(appContext, serviceClass, intent)
        }
    }
}

private fun Intent.putPushMessage(message: PushMessage) {
    putExtra(PushMessageService.EXTRA_TITLE, message.title)
    putExtra(PushMessageService.EXTRA_CONTENT, message.content)
    putExtra(PushMessageService.EXTRA_PAYLOAD, message.payload)
    putExtra(PushMessageService.EXTRA_MESSAGE_ID, message.messageId)
    putExtra(PushMessageService.EXTRA_PASS_THROUGH, message.passThrough)
    putExtra(
        PushMessageService.EXTRA_EXTRAS,
        android.os.Bundle().apply {
            message.extras.forEach { (key, value) -> putString(key, value) }
        },
    )
}

internal object PushMessageServiceDeliverer {
    fun deliver(
        context: Context,
        serviceClass: Class<out PushMessageService>,
        intent: Intent,
    ) {
        runCatching {
            val service = serviceClass.getDeclaredConstructor().newInstance()
            service.deliverPushEvent(context, intent)
        }.onFailure {
            runCatching {
                @Suppress("DEPRECATION")
                context.startService(intent)
            }
        }
    }
}
