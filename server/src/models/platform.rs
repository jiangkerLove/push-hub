/// 自建在线推送通道，对应 Android 端 `PushVendor.ONLINE`。
pub const FALLBACK_PLATFORM: &str = "online";

/// 已接入的厂商推送平台（不含 online）
pub fn is_vendor_platform(platform: &str, configured: &[String]) -> bool {
    let normalized = platform.trim().to_lowercase();
    normalized != FALLBACK_PLATFORM && configured.iter().any(|p| p == &normalized)
}

/// 将平台名规范为小写；未配置适配器时走在线兜底通道。
pub fn resolve_delivery_platform(platform: &str, configured: &[String]) -> String {
    let normalized = platform.trim().to_lowercase();
    if configured.iter().any(|p| p == &normalized) {
        normalized
    } else {
        FALLBACK_PLATFORM.to_string()
    }
}
