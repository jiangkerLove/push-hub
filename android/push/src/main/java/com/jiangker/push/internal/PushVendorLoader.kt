package com.jiangker.push.internal

import android.content.Context
import android.util.Log
import com.jiangker.push.core.ManufacturerDetector
import com.jiangker.push.core.PushConfig
import com.jiangker.push.core.PushProvider
import com.jiangker.push.core.PushVendor

/**
 * 按当前设备厂商与 [PushConfig] 解析唯一需要初始化的厂商通道。
 *
 * 各厂商实现位于独立模块（如 `:push-huawei`），由业务方按需添加依赖；
 * 此处通过反射加载，未集成对应模块时返回空列表并走在线推送。
 */
internal object PushVendorLoader {
    private const val TAG = "PushHub-VendorLoader"

    private val providerClassNames = mapOf(
        PushVendor.XIAOMI to "com.jiangker.push.xiaomi.XiaomiPushProvider",
        PushVendor.HUAWEI to "com.jiangker.push.huawei.HuaweiPushProvider",
        PushVendor.OPPO to "com.jiangker.push.oppo.OppoPushProvider",
        PushVendor.VIVO to "com.jiangker.push.vivo.VivoPushProvider",
        PushVendor.HONOR to "com.jiangker.push.honor.HonorPushProvider",
        PushVendor.MEIZU to "com.jiangker.push.meizu.MeizuPushProvider",
    )

    fun resolve(context: Context, config: PushConfig): List<PushProvider> {
        val vendor = ManufacturerDetector.detect()
        if (vendor == null) {
            Log.i(TAG, "unknown manufacturer, no vendor provider")
            return emptyList()
        }
        if (!isVendorConfigured(vendor, config)) {
            Log.i(TAG, "vendor ${vendor.name.lowercase()} not enabled in PushHubConfig")
            return emptyList()
        }

        val provider = createProvider(vendor)
        if (provider == null) {
            Log.w(
                TAG,
                "vendor ${vendor.name.lowercase()} SDK not integrated; " +
                    "add implementation(project(\":push-${vendor.name.lowercase()}\"))",
            )
            return emptyList()
        }
        if (!provider.isSupported(context)) {
            Log.w(TAG, "provider ${vendor.name.lowercase()} not supported on this device")
            return emptyList()
        }

        Log.i(TAG, "resolved provider: ${vendor.name.lowercase()}")
        return listOf(provider)
    }

    private fun isVendorConfigured(vendor: PushVendor, config: PushConfig): Boolean = when (vendor) {
        PushVendor.XIAOMI -> config.xiaomi != null
        PushVendor.HUAWEI -> config.huawei != null
        PushVendor.OPPO -> config.oppo != null
        PushVendor.VIVO -> config.vivo != null
        PushVendor.HONOR -> config.honor != null
        PushVendor.MEIZU -> config.meizu != null
        PushVendor.FCM, PushVendor.ONLINE -> false
    }

    private fun createProvider(vendor: PushVendor): PushProvider? {
        val className = providerClassNames[vendor] ?: return null
        return runCatching {
            Class.forName(className)
                .getDeclaredConstructor()
                .newInstance() as PushProvider
        }.getOrNull()
    }
}
