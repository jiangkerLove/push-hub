package com.jiangker.push

import android.content.Context
import com.jiangker.push.core.DeviceRegistrar
import com.jiangker.push.core.HuaweiConfig
import com.jiangker.push.core.HonorConfig
import com.jiangker.push.core.MeizuConfig
import com.jiangker.push.core.OppoConfig
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushManager
import com.jiangker.push.core.PushRegistrationListener
import com.jiangker.push.core.PushServerConfig
import com.jiangker.push.core.PushVendor
import com.jiangker.push.core.VivoConfig
import com.jiangker.push.core.XiaomiConfig
import com.jiangker.push.internal.PushVendorLoader

/**
 * Push Hub 对外统一入口，第三方项目只需依赖 `:push` 模块并调用 [init]。
 *
 * 各厂商通道为独立模块，按需添加依赖；运行时识别设备厂商后只初始化已集成的对应通道。
 */
object PushHub {
    fun init(context: Context, config: PushHubConfig) {
        val pushConfig = PushConfig(
            xiaomi = config.xiaomiAppId?.let { appId ->
                XiaomiConfig(
                    appId = appId,
                    appKey = config.xiaomiAppKey.orEmpty(),
                )
            },
            huawei = config.huaweiAppId?.let { appId ->
                HuaweiConfig(appId = appId)
            },
            oppo = config.oppoAppKey?.let { appKey ->
                OppoConfig(
                    appKey = appKey,
                    appSecret = config.oppoAppSecret.orEmpty(),
                )
            },
            vivo = config.vivoAppId?.let { appId ->
                VivoConfig(
                    appId = appId,
                    appKey = config.vivoAppKey.orEmpty(),
                )
            },
            honor = config.honorAppId?.let { appId ->
                HonorConfig(appId = appId)
            },
            meizu = config.meizuAppId?.let { appId ->
                MeizuConfig(
                    appId = appId,
                    appKey = config.meizuAppKey.orEmpty(),
                )
            },
            server = PushServerConfig(
                baseUrl = config.serverBaseUrl,
                hubAppId = config.pushHubAppId,
                packageName = context.packageName,
            ),
        )
        val providers = PushVendorLoader.resolve(context, pushConfig)
        PushManager.init(
            context = context,
            config = pushConfig,
            messageService = config.messageService,
            providers = providers,
            registrationListener = config.registrationListener,
        )
    }

    fun setRegistrationListener(listener: PushRegistrationListener?) {
        DeviceRegistrar.setRegistrationListener(listener)
    }

    fun getDeviceId(): String? = DeviceRegistrar.getPrimaryDeviceId()

    fun getDeviceId(vendor: PushVendor): String? = DeviceRegistrar.getDeviceId(vendor)

    fun getDeviceId(platform: String): String? = DeviceRegistrar.getDeviceId(platform)

    fun unregisterAll() {
        PushManager.unregisterAll()
    }
}
