package com.jiangker.push.core

import android.os.Build

object ManufacturerDetector {
    fun detect(): PushVendor? {
        val manufacturer = Build.MANUFACTURER.lowercase()
        val brand = Build.BRAND.lowercase()
        return when {
            manufacturer.contains("xiaomi") ||
                manufacturer.contains("redmi") ||
                brand.contains("xiaomi") ||
                brand.contains("redmi") -> PushVendor.XIAOMI

            manufacturer.contains("huawei") || brand.contains("huawei") -> PushVendor.HUAWEI
            manufacturer.contains("honor") || brand.contains("honor") -> PushVendor.HONOR
            // Heytap Push covers OPPO / realme / OnePlus
            manufacturer.contains("oppo") ||
                manufacturer.contains("realme") ||
                manufacturer.contains("oneplus") ||
                brand.contains("oppo") ||
                brand.contains("realme") ||
                brand.contains("oneplus") -> PushVendor.OPPO
            manufacturer.contains("vivo") ||
                manufacturer.contains("iqoo") ||
                brand.contains("vivo") ||
                brand.contains("iqoo") -> PushVendor.VIVO
            manufacturer.contains("meizu") || brand.contains("meizu") -> PushVendor.MEIZU
            else -> null
        }
    }
}
