use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterDeviceRequest {
    /// Push Hub 控制台中的应用 ID（推荐必填，用于校验设备归属）
    #[serde(default)]
    pub app_id: Option<String>,
    /// 客户端已绑定的 device_id；厂商 push_token 刷新时传入可保持身份稳定
    #[serde(default, alias = "id")]
    pub device_id: Option<String>,
    /// 客户端平台标识：Android 为包名，iOS 为 Bundle ID，鸿蒙为 Bundle Name
    pub package_name: String,
    /// 厂商平台名（xiaomi / huawei / online），或纯在线设备的 online
    pub platform: String,
    /// 对外主 token：厂商 regId；纯在线设备时为轮询 token
    pub push_token: String,
    /// 内部在线轮询 token，仅随厂商设备一并上报，不单独创建设备
    #[serde(default)]
    pub online_token: Option<String>,
}

impl RegisterDeviceRequest {
    pub fn normalized_online_token(&self) -> Option<String> {
        self.online_token
            .as_ref()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(str::to_string)
    }
}
