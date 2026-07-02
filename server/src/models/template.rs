use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{ClickAction, SendPushRequest};
use crate::AppError;
use crate::AppResult;

/// 各推送平台的通道/分类配置，在后台为每个模板单独配置。
/// 小米：通知渠道 channel_id；
/// OPPO：消息分类 category，非 IM 私信另需 private_template_id（官方审核模板）；
/// 华为：Push Kit `category`；
/// vivo：消息二级分类 `category`；
/// 魅族：`msg_type`（PUBLIC / PRIVATE）；
/// 荣耀：无需配置；`honor` 仅作 API 可选覆盖，未填时发送端默认 `NORMAL`。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateChannels {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xiaomi: Option<XiaomiChannelConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub huawei: Option<HuaweiCategoryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oppo: Option<OppoChannelConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vivo: Option<VivoCategoryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub honor: Option<HuaweiCategoryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meizu: Option<MeizuMsgTypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XiaomiChannelConfig {
    pub channel_id: String,
}

/// OPPO 消息分类，以及与我方拼接模板的对应关系。
/// 非 IM 私信：`private_template_id` 指向 OPPO 审核模板，与本模板 `{{变量}}` 一一对应；
/// 发送时 OPPO 转成模板 ID + 参数，其它厂商用已拼接的 title/body。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OppoChannelConfig {
    /// 消息分类：NEWS / MARKETING / IM / ORDER 等
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Android 通知渠道 ID，低版本兼容；缺省时回退为 category
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// 对应的 OPPO 审核私信模板 ID（非 IM 私信必填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_template_id: Option<String>,
}

/// vivo 消息二级分类，对应推送 API `category`（大写，如 IM、MARKETING）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivoCategoryConfig {
    pub category: String,
}

/// 华为 / 荣耀消息分类，对应 Push Kit `android.notification.category`
///（MARKETING / IM / WORK / EXPRESS 等，大写字符串）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuaweiCategoryConfig {
    pub category: String,
}

/// 魅族消息分类，对应 `noticeBarInfo.noticeMsgType`：
/// PUBLIC=公信(0)，PRIVATE=私信(1)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeizuMsgTypeConfig {
    pub msg_type: String,
}

impl TemplateChannels {
    pub fn xiaomi_channel_id(&self) -> Option<&str> {
        self.xiaomi
            .as_ref()
            .map(|c| c.channel_id.as_str())
            .filter(|id| !id.is_empty())
    }

    pub fn huawei_category(&self) -> Option<&str> {
        self.huawei
            .as_ref()
            .map(|c| c.category.as_str())
            .filter(|id| !id.is_empty())
    }

    pub fn oppo_category(&self) -> Option<&str> {
        self.oppo
            .as_ref()
            .and_then(|c| c.category.as_deref())
            .map(str::trim)
            .filter(|id| !id.is_empty())
    }

    /// Android 通知渠道 ID（旧版兼容）。不要回退到 category：消息分类与通知渠道不是同一个值。
    pub fn oppo_channel_id(&self) -> Option<&str> {
        self.oppo
            .as_ref()
            .and_then(|c| c.channel_id.as_deref())
            .map(str::trim)
            .filter(|id| !id.is_empty())
    }

    /// Android 通知渠道；若与 category 相同（旧数据误存）则忽略。
    pub fn oppo_push_channel_id(&self) -> Option<&str> {
        let channel_id = self.oppo_channel_id()?;
        if let Some(category) = self.oppo_category() {
            if channel_id.eq_ignore_ascii_case(category) {
                return None;
            }
        }
        Some(channel_id)
    }

    pub fn oppo_private_template_id(&self) -> Option<&str> {
        self.oppo
            .as_ref()
            .and_then(|c| c.private_template_id.as_deref())
            .map(str::trim)
            .filter(|id| !id.is_empty())
    }

    /// 非 IM 私信且未绑定 OPPO 审核模板时须补齐。
    /// 发送主路径看模板映射；分类仅用于标出 IM（可不绑模板）。
    pub fn oppo_requires_private_template(&self) -> bool {
        if self.oppo_private_template_id().is_some() {
            return false;
        }
        match self.oppo_category().map(|c| c.to_uppercase()) {
            Some(c) => matches!(
                c.as_str(),
                "ACCOUNT" | "DEVICE_REMINDER" | "ORDER" | "TODO" | "SUBSCRIPTION"
            ),
            None => false,
        }
    }

    /// 可否走 OPPO 厂商：已对应审核模板，或 IM / 公信分类，或兼容旧 channel_id。
    pub fn oppo_can_send(&self) -> bool {
        if self.oppo_private_template_id().is_some() {
            return true;
        }
        match self.oppo_category().map(|c| c.to_uppercase()) {
            Some(c) => matches!(
                c.as_str(),
                "IM" | "NEWS" | "CONTENT" | "MARKETING" | "SOCIAL"
            ),
            None => self
                .oppo
                .as_ref()
                .and_then(|c| c.channel_id.as_deref())
                .map(str::trim)
                .is_some_and(|id| !id.is_empty()),
        }
    }

    /// 非 IM 私信分类须绑定 OPPO 审核模板。
    pub fn validate_oppo_private_template(&self) -> AppResult<()> {
        if self.oppo_requires_private_template() && self.oppo_private_template_id().is_none() {
            return Err(AppError::BadRequest(
                "OPPO non-IM private messages require channels.oppo.private_template_id".into(),
            ));
        }
        Ok(())
    }

    /// 可选覆盖；控制台与模板均不要求配置，发送时未填则用 `NORMAL`。
    pub fn honor_category(&self) -> Option<&str> {
        self.honor
            .as_ref()
            .map(|c| c.category.as_str())
            .filter(|id| !id.is_empty())
    }

    pub fn vivo_category(&self) -> Option<&str> {
        self.vivo
            .as_ref()
            .map(|c| c.category.as_str())
            .filter(|id| !id.is_empty())
    }

    pub fn meizu_msg_type(&self) -> Option<&str> {
        self.meizu
            .as_ref()
            .map(|c| c.msg_type.as_str())
            .filter(|id| !id.is_empty())
    }

    /// 魅族 `noticeMsgType`：0 公信，1 私信。
    pub fn meizu_notice_msg_type(&self) -> Option<i64> {
        match self.meizu_msg_type()?.trim().to_uppercase().as_str() {
            "PUBLIC" | "0" => Some(0),
            "PRIVATE" | "1" => Some(1),
            _ => None,
        }
    }
}

pub const DEFAULT_MESSAGE_CACHE_DAYS: i64 = 7;

/// 发送请求 notify_id 合法范围：0~2147483647。
pub fn normalize_notify_id(value: Option<i32>) -> AppResult<Option<i32>> {
    match value {
        None => Ok(None),
        Some(id) if (0..=2_147_483_647).contains(&id) => Ok(Some(id)),
        Some(id) => Err(AppError::BadRequest(format!(
            "notify_id must be an integer from 0 to 2147483647, got {id}"
        ))),
    }
}

fn default_message_cache_days() -> i64 {
    DEFAULT_MESSAGE_CACHE_DAYS
}

/// 模板类型：公信（厂商通道，发送时填写标题内容）/ 私信（可预设文案与变量）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    /// 私信模板：可预设标题、正文与变量
    #[default]
    Private,
    /// 公信模板：仅配置通道与行为，发送时再填标题、内容
    Public,
}

impl TemplateKind {
    pub fn from_db(value: &str) -> Self {
        match value.trim() {
            "public" => Self::Public,
            _ => Self::Private,
        }
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }

    pub fn is_public(self) -> bool {
        matches!(self, Self::Public)
    }
}

/// 私信模板内容模式：自由填写（发送时再填标题内容）/ 拼接（预设文案与 {{变量}}）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemplateContentMode {
    /// 发送时填写标题与内容，模板仅配置通道与行为
    Free,
    /// 预设标题、正文，发送时填写变量拼接
    #[default]
    Compose,
}

impl TemplateContentMode {
    pub fn from_db(value: &str) -> Self {
        match value.trim() {
            "free" => Self::Free,
            _ => Self::Compose,
        }
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Compose => "compose",
        }
    }

    pub fn is_free(self) -> bool {
        matches!(self, Self::Free)
    }
}

fn default_content_mode_db() -> String {
    TemplateContentMode::Compose.as_db().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushTemplate {
    pub id: String,
    pub app_id: String,
    pub name: String,
    /// `public` 公信 / `private` 私信
    #[serde(default = "default_template_kind_db")]
    pub kind: String,
    /// 私信内容模式：`free` 自由填写 / `compose` 拼接变量
    #[serde(default = "default_content_mode_db")]
    pub content_mode: String,
    pub title: String,
    pub body: String,
    pub channels: sqlx::types::Json<TemplateChannels>,
    /// 历史字段，仅兼容旧库；点击行为以发送请求为准，不再由模板限定。
    #[serde(default, skip_serializing)]
    pub click_action: sqlx::types::Json<ClickAction>,
    /// 在线消息默认缓存天数（自发送时起算），发送时可被 cache_until 覆盖。
    #[serde(default = "default_message_cache_days")]
    pub message_cache_days: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_template_kind_db() -> String {
    TemplateKind::Private.as_db().to_string()
}

impl PushTemplate {
    pub fn template_kind(&self) -> TemplateKind {
        TemplateKind::from_db(&self.kind)
    }

    pub fn is_public(&self) -> bool {
        self.template_kind().is_public()
    }

    pub fn content_mode(&self) -> TemplateContentMode {
        TemplateContentMode::from_db(&self.content_mode)
    }

    pub fn is_private_free(&self) -> bool {
        !self.is_public() && self.content_mode().is_free()
    }

    pub fn is_private_compose(&self) -> bool {
        !self.is_public() && !self.content_mode().is_free()
    }
}

/// 解析本次推送的在线消息缓存截止时间：发送参数优先，否则按模板默认天数。
pub fn resolve_message_cache_until(
    template: Option<&PushTemplate>,
    request: &SendPushRequest,
) -> AppResult<DateTime<Utc>> {
    if let Some(until) = request.cache_until {
        if until <= Utc::now() {
            return Err(AppError::BadRequest(
                "cache_until must be in the future".into(),
            ));
        }
        return Ok(until);
    }
    let days = template
        .map(|item| item.message_cache_days)
        .unwrap_or(DEFAULT_MESSAGE_CACHE_DAYS)
        .max(1);
    Ok(Utc::now() + Duration::days(days))
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub kind: TemplateKind,
    #[serde(default)]
    pub content_mode: TemplateContentMode,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub channels: TemplateChannels,
    #[serde(default = "default_message_cache_days")]
    pub message_cache_days: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub kind: TemplateKind,
    #[serde(default)]
    pub content_mode: TemplateContentMode,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub channels: TemplateChannels,
    #[serde(default = "default_message_cache_days")]
    pub message_cache_days: i64,
}

pub fn validate_template_fields(
    name: &str,
    kind: TemplateKind,
    content_mode: TemplateContentMode,
    title: &str,
    body: &str,
) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if kind.is_public() {
        return Ok(());
    }
    if content_mode.is_free() {
        return Ok(());
    }
    if title.trim().is_empty() || body.trim().is_empty() {
        return Err(AppError::BadRequest(
            "title and body are required for compose private templates".into(),
        ));
    }
    Ok(())
}
