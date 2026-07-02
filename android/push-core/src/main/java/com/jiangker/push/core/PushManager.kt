package com.jiangker.push.core

import android.content.Context
import android.util.Log

object PushManager {
    private const val TAG = "PushHub-Manager"

    private var initialized = false
    private val activeProviders = mutableListOf<PushProvider>()

    fun init(
        context: Context,
        config: PushConfig,
        messageService: Class<out PushMessageService>,
        providers: List<PushProvider>,
        registrationListener: PushRegistrationListener? = null,
    ) {
        if (initialized) {
            Log.w(TAG, "init skipped: already initialized")
            return
        }
        if (!ProcessUtils.isMainProcess(context)) {
            Log.i(TAG, "init skipped: not main process")
            return
        }

        PushEventDispatcher.init(context, messageService)

        config.server?.let { server ->
            DeviceRegistrar.init(context, server, registrationListener)
            DeviceRegistrar.startOnlinePolling()
        }

        Log.i(
            TAG,
            "initializing providers: ${providers.joinToString { it.vendor.name.lowercase() }.ifBlank { "<none>" }}",
        )
        providers.forEach { provider ->
            provider.init(context.applicationContext, config)
            provider.register()
            activeProviders += provider
        }

        if (config.server != null && providers.isEmpty()) {
            Log.w(TAG, "no vendor provider matched, registering online-only device")
            DeviceRegistrar.registerOnlineOnlyDevice()
        } else if (config.server != null && providers.isNotEmpty()) {
            scheduleVendorRegistrationBootstrap()
        }

        initialized = true
        Log.i(TAG, "init complete")
    }

    private fun scheduleVendorRegistrationBootstrap() {
        android.os.Handler(android.os.Looper.getMainLooper()).postDelayed({
            if (!DeviceRegistrar.hasRegisteredDevice()) {
                Log.w(
                    TAG,
                    "vendor token not registered yet, bootstrap online-only device for delivery",
                )
                DeviceRegistrar.registerOnlineOnlyDevice()
            }
        }, VENDOR_REGISTRATION_BOOTSTRAP_MS)
    }

    private const val VENDOR_REGISTRATION_BOOTSTRAP_MS = 5_000L

    fun unregisterAll() {
        OnlinePushConnection.stop()
        activeProviders.forEach { it.unregister() }
        activeProviders.clear()
        initialized = false
    }
}
