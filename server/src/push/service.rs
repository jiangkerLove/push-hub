use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::db::Database;
use crate::models::{
    DeliveryMode, Device, PlatformSendResult, PushFallbackTarget, PushTargets, RenderedNotification,
    SendPushRequest, SendPushResponse, TemplateChannels, FALLBACK_PLATFORM,
    HuaweiCategoryConfig, MeizuMsgTypeConfig, XiaomiChannelConfig, is_vendor_platform,
    normalize_notify_id,
    resolve_delivery_platform, resolve_message_cache_until, OppoChannelConfig, VivoCategoryConfig,
};
use crate::push::template_render::render_template;
use crate::push::ProviderSendResult;
use crate::push::PushHub;
use crate::AppError;
use crate::AppResult;

#[derive(Debug, Clone)]
struct TargetDevice {
    device_id: Option<String>,
    platform: String,
    push_token: String,
    package_name: String,
    online_token: Option<String>,
    last_online_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TargetDevice {
    fn is_recently_online(&self, ttl_secs: i64) -> bool {
        self.last_online_at.is_some_and(|at| {
            chrono::Utc::now().signed_duration_since(at).num_seconds() < ttl_secs
        })
    }
}

pub struct PushService {
    config: Arc<Config>,
}

impl PushService {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn send_with(
        &self,
        db: &Database,
        hub: Arc<PushHub>,
        app_id: Option<&str>,
        package_name: String,
        _online_push_fallback_secs: i64,
        request: SendPushRequest,
    ) -> AppResult<SendPushResponse> {
        let targets = self
            .resolve_targets(db, &request.targets, app_id, &package_name)
            .await?;
        if targets.is_empty() {
            return Err(AppError::BadRequest("no valid push targets".into()));
        }

        let notification = self
            .build_notification(db, &request, &package_name, app_id)
            .await?;
        let configured = hub.platforms();

        let mut stats: HashMap<String, PlatformSendResult> = HashMap::new();

        for target in targets {
            Self::dispatch_target(
                db,
                &hub,
                &notification,
                &target,
                &configured,
                &mut stats,
            )
            .await?;
        }

        let platform_results: Vec<PlatformSendResult> = stats.into_values().collect();
        let total_success: usize = platform_results.iter().map(|r| r.success).sum();
        let total_failed: usize = platform_results.iter().map(|r| r.failed).sum();

        Ok(SendPushResponse {
            total: total_success + total_failed,
            success: total_success,
            failed: total_failed,
            platforms: platform_results,
            job_id: None,
        })
    }

    pub async fn send_traced(
        &self,
        db: Arc<Database>,
        hub: Arc<PushHub>,
        app_id: &str,
        package_name: String,
        online_push_fallback_secs: i64,
        request: SendPushRequest,
    ) -> AppResult<SendPushResponse> {
        let _ = online_push_fallback_secs;
        use crate::push::trace::PushTracer;

        let targets = self
            .resolve_targets(db.as_ref(), &request.targets, Some(app_id), &package_name)
            .await?;
        if targets.is_empty() {
            return Err(AppError::BadRequest("no valid push targets".into()));
        }

        let notification = self
            .build_notification(db.as_ref(), &request, &package_name, Some(app_id))
            .await?;

        let (trace_template_id, trace_template_name) = if request.uses_template() {
            let tid = request.template_id.as_deref().unwrap_or("").trim();
            let template = db
                .templates()
                .find_by_id(tid)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("template not found: {tid}")))?;
            (template.id.clone(), template.name.clone())
        } else {
            (String::new(), "直接推送".to_string())
        };

        let tracer = PushTracer::start(
            db.clone(),
            app_id,
            &trace_template_id,
            &trace_template_name,
            &notification.title,
            &notification.body,
            targets.len() as i64,
        )
        .await?;
        tracer.template_rendered(&notification.title, &notification.body).await?;

        let configured = hub.platforms();
        let mut stats: HashMap<String, PlatformSendResult> = HashMap::new();

        for target in targets {
            let target_id = tracer
                .start_target(
                    target.device_id.as_deref(),
                    &target.platform,
                    &target.push_token,
                )
                .await?;

            let vendor_platform = resolve_delivery_platform(&target.platform, &configured);
            let has_vendor = can_use_vendor(&target.platform, &configured, &notification);

            if !has_vendor {
                tracer
                    .route_selected(&target_id, "online_cache", "online", false)
                    .await?;
                let token = online_delivery_token(&target);
                match Self::send_via_platform(
                    &hub,
                    FALLBACK_PLATFORM,
                    &[token],
                    &notification,
                    target.package_name.clone(),
                    None,
                    &mut stats,
                )
                .await
                {
                    Ok(result) => {
                        tracer.record_online_result(&target_id, &result).await?;
                    }
                    Err(err) => {
                        tracer
                            .record_vendor_result(
                                &target_id,
                                FALLBACK_PLATFORM,
                                &ProviderSendResult {
                                    success_count: 0,
                                    failure_count: 1,
                                    message_id: None,
                                    outbox_ids: vec![],
                                    ws_delivered: 0,
                                },
                                &err.to_string(),
                            )
                            .await?;
                        tracing::warn!(error = %err, "online cache enqueue failed");
                    }
                }
                continue;
            }

            let online_token = target
                .online_token
                .as_ref()
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_string());

            if let Some(online_token) = online_token {
                tracer
                    .route_selected(&target_id, "online_first", &target.platform, false)
                    .await?;
                let online_rendered =
                    with_vendor_fallback(&notification, &target, &vendor_platform);
                match Self::send_via_platform(
                    &hub,
                    FALLBACK_PLATFORM,
                    &[online_token],
                    &online_rendered,
                    target.package_name.clone(),
                    None,
                    &mut stats,
                )
                .await
                {
                    Ok(result) => {
                        tracer.record_online_result(&target_id, &result).await?;
                        if let Some((vendor_platform, fallback_result)) =
                            Self::handle_online_vendor_fallback(
                                db.as_ref(),
                                &hub,
                                &online_rendered,
                                &result,
                            )
                            .await?
                        {
                            tracer
                                .route_selected(
                                    &target_id,
                                    "vendor_fallback",
                                    &vendor_platform,
                                    false,
                                )
                                .await?;
                            let detail = if fallback_result.success_count > 0 {
                                "WebSocket 未送达，已立即降级厂商离线推送"
                            } else {
                                fallback_result
                                    .last_error
                                    .as_deref()
                                    .unwrap_or("厂商降级推送失败")
                            };
                            tracer
                                .record_vendor_result(
                                    &target_id,
                                    &vendor_platform,
                                    &fallback_provider_result(&fallback_result),
                                    detail,
                                )
                                .await?;
                        }
                        continue;
                    }
                    Err(err) => {
                        tracer
                            .route_selected(&target_id, "vendor_fallback", &vendor_platform, false)
                            .await?;
                        tracing::warn!(error = %err, "online push failed, falling back to vendor");
                    }
                }
            } else {
                tracer
                    .route_selected(&target_id, "vendor_direct", &vendor_platform, false)
                    .await?;
            }

            let result = Self::send_via_platform(
                &hub,
                &vendor_platform,
                &[target.push_token.clone()],
                &notification,
                target.package_name.clone(),
                Some(DeliveryMode::Notification),
                &mut stats,
            )
            .await;

            match result {
                Ok(provider_result) => {
                    tracer
                        .record_vendor_result(
                            &target_id,
                            &vendor_platform,
                            &provider_result,
                            "厂商通道推送完成",
                        )
                        .await?;
                }
                Err(err) => {
                    tracer
                        .record_vendor_result(
                            &target_id,
                            &vendor_platform,
                            &ProviderSendResult {
                                success_count: 0,
                                failure_count: 1,
                                message_id: None,
                                outbox_ids: vec![],
                                ws_delivered: 0,
                            },
                            &err.to_string(),
                        )
                        .await?;
                    tracing::warn!(error = %err, "vendor push failed for target");
                }
            }
        }

        let platform_results: Vec<PlatformSendResult> = stats.into_values().collect();
        let total_success: usize = platform_results.iter().map(|r| r.success).sum();
        let total_failed: usize = platform_results.iter().map(|r| r.failed).sum();
        tracer
            .finish_job(total_success as i64, total_failed as i64)
            .await?;

        Ok(SendPushResponse {
            total: total_success + total_failed,
            success: total_success,
            failed: total_failed,
            platforms: platform_results,
            job_id: Some(tracer.job_id().to_string()),
        })
    }

    async fn dispatch_target(
        db: &Database,
        hub: &PushHub,
        notification: &RenderedNotification,
        target: &TargetDevice,
        configured: &[String],
        stats: &mut HashMap<String, PlatformSendResult>,
    ) -> AppResult<()> {
        let vendor_platform = resolve_delivery_platform(&target.platform, configured);
        let has_vendor = can_use_vendor(&target.platform, configured, notification);

        // 未接入厂商或未配置推送通道：只写入 outbox，不尝试厂商离线
        if !has_vendor {
            let token = online_delivery_token(target);
            return Self::send_via_platform(
                hub,
                FALLBACK_PLATFORM,
                &[token],
                notification,
                target.package_name.clone(),
                None,
                stats,
            )
            .await
            .map(|_| ());
        }

        // 有厂商通道：优先在线；WebSocket 未送达则立即降级厂商
        let online_token = target
            .online_token
            .as_ref()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string());

        if let Some(online_token) = online_token {
            let online_rendered = with_vendor_fallback(notification, target, &vendor_platform);
            match Self::send_via_platform(
                hub,
                FALLBACK_PLATFORM,
                &[online_token],
                &online_rendered,
                target.package_name.clone(),
                None,
                stats,
            )
            .await
            {
                Ok(result) => {
                    if let Some((_, fallback_result)) = Self::handle_online_vendor_fallback(
                        db,
                        hub,
                        &online_rendered,
                        &result,
                    )
                    .await?
                    {
                        if fallback_result.failure_count > 0 {
                            tracing::warn!(
                                platform = %target.platform,
                                error = fallback_result.last_error.as_deref().unwrap_or("vendor fallback failed"),
                                "online push enqueued but vendor fallback failed",
                            );
                        }
                    }
                    return Ok(());
                }
                Err(err) => {
                    tracing::warn!(
                        platform = %target.platform,
                        error = %err,
                        "online enqueue failed, falling back to vendor offline immediately",
                    );
                }
            }
        }

        Self::send_via_platform(
            hub,
            &vendor_platform,
            &[target.push_token.clone()],
            notification,
            target.package_name.clone(),
            Some(DeliveryMode::Notification),
            stats,
        )
        .await
        .map(|_| ())
    }

    async fn handle_online_vendor_fallback(
        db: &Database,
        hub: &PushHub,
        rendered: &RenderedNotification,
        result: &ProviderSendResult,
    ) -> AppResult<Option<(String, crate::push::fallback::FallbackBatchResult)>> {
        let Some(fallback) = rendered.vendor_fallback.as_ref() else {
            return Ok(None);
        };
        if result.outbox_ids.is_empty() {
            return Ok(None);
        }

        if result.ws_delivered == 0 {
            let fallback_result =
                crate::push::fallback::fallback_outbox_ids(db, hub, &result.outbox_ids).await?;
            Ok(Some((fallback.platform.clone(), fallback_result)))
        } else {
            // 保留 fallback 信息，等待客户端 ACK；超时后由 fallback worker 降级厂商
            Ok(None)
        }
    }

    async fn send_via_platform(
        hub: &PushHub,
        delivery_platform: &str,
        tokens: &[String],
        notification: &RenderedNotification,
        package_name: String,
        force_delivery_mode: Option<DeliveryMode>,
        stats: &mut HashMap<String, PlatformSendResult>,
    ) -> AppResult<ProviderSendResult> {
        let provider = hub.get(delivery_platform).ok_or_else(|| {
            AppError::Push(format!(
                "delivery platform '{delivery_platform}' is not registered"
            ))
        })?;

        let rendered = RenderedNotification {
            title: notification.title.clone(),
            body: notification.body.clone(),
            payload: notification.payload.clone(),
            click_action: notification.click_action.clone(),
            package_name,
            channels: notification.channels.clone(),
            delivery_mode: force_delivery_mode.unwrap_or(notification.delivery_mode),
            notify_id: notification.notify_id,
            vendor_fallback: notification.vendor_fallback.clone(),
            expires_at: notification.expires_at,
            title_variables: notification.title_variables.clone(),
            body_variables: notification.body_variables.clone(),
        };

        let result = provider.send(tokens, &rendered).await;
        let entry = stats
            .entry(delivery_platform.to_string())
            .or_insert_with(|| PlatformSendResult {
                platform: delivery_platform.to_string(),
                success: 0,
                failed: 0,
                message_id: None,
            });
        match result {
            Ok(provider_result) => {
                entry.success += provider_result.success_count;
                entry.failed += provider_result.failure_count;
                if provider_result.message_id.is_some() {
                    entry.message_id = provider_result.message_id.clone();
                }
                Ok(provider_result)
            }
            Err(err) => {
                entry.failed += tokens.len();
                Err(err)
            }
        }
    }

    async fn build_notification(
        &self,
        db: &Database,
        request: &SendPushRequest,
        package_name: &str,
        app_id: Option<&str>,
    ) -> AppResult<RenderedNotification> {
        if request.uses_template() {
            let template_id = request.template_id.as_deref().unwrap_or("").trim();
            let template = db
                .templates()
                .find_by_id(template_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("template not found: {template_id}")))?;

            if let Some(aid) = app_id {
                if !template.app_id.is_empty() && template.app_id != aid {
                    return Err(AppError::BadRequest(
                        "template does not belong to this app".into(),
                    ));
                }
            }

            let (title, body) = if template.is_public() {
                let title = request
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "title is required when using a public template".into(),
                        )
                    })?
                    .to_string();
                let body = request
                    .body
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "body is required when using a public template".into(),
                        )
                    })?
                    .to_string();
                (title, body)
            } else if template.is_private_free() {
                let title = request
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "title is required when using a free private template".into(),
                        )
                    })?
                    .to_string();
                let body = request
                    .body
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "body is required when using a free private template".into(),
                        )
                    })?
                    .to_string();
                (title, body)
            } else {
                // 拼接模式：先按模版自动拼出正文，供非 OPPO 厂商与在线使用；
                // 变量表仍保留，OPPO 走模板 ID + 参数转换。
                render_template(
                    &template,
                    &request.title_variables,
                    &request.body_variables,
                )?
            };

            let channels = merge_channels(&template.channels.0, &request.channels);
            let channels = if let Some(aid) = app_id {
                fill_missing_channels(db, aid, channels).await?
            } else {
                channels
            };
            // 点击行为跟本次消息内容相关，不由模板限定
            let click_action = request
                .click_action
                .clone()
                .unwrap_or_default();
            click_action
                .validate()
                .map_err(AppError::BadRequest)?;

            let expires_at = resolve_message_cache_until(Some(&template), request)?;
            let notify_id = normalize_notify_id(request.notify_id)?;

            channels.validate_oppo_private_template()?;

            return Ok(RenderedNotification {
                title,
                body,
                payload: request.payload.clone(),
                click_action,
                package_name: package_name.to_string(),
                channels,
                delivery_mode: request.delivery_mode,
                notify_id,
                vendor_fallback: None,
                expires_at,
                title_variables: request.title_variables.clone(),
                body_variables: request.body_variables.clone(),
            });
        }

        let title = request
            .title
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest("title is required for private direct push".into())
            })?
            .to_string();
        let body = request
            .body
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest("body is required for private direct push".into())
            })?
            .to_string();

        let channels = if request.channels.xiaomi.is_some()
            || request.channels.huawei.is_some()
            || request.channels.oppo.is_some()
            || request.channels.vivo.is_some()
            || request.channels.honor.is_some()
            || request.channels.meizu.is_some()
        {
            request.channels.clone()
        } else if let Some(app_id) = app_id {
            default_channels_for_app(db, app_id).await?
        } else {
            TemplateChannels::default()
        };

        let click_action = request
            .click_action
            .clone()
            .unwrap_or_default();
        click_action
            .validate()
            .map_err(AppError::BadRequest)?;

        let expires_at = resolve_message_cache_until(None, request)?;
        let notify_id = normalize_notify_id(request.notify_id)?;

        channels.validate_oppo_private_template()?;

        Ok(RenderedNotification {
            title,
            body,
            payload: request.payload.clone(),
            click_action,
            package_name: package_name.to_string(),
            channels,
            delivery_mode: request.delivery_mode,
            notify_id,
            vendor_fallback: None,
            expires_at,
            title_variables: HashMap::new(),
            body_variables: HashMap::new(),
        })
    }

    async fn resolve_targets(
        &self,
        db: &Database,
        targets: &PushTargets,
        app_id: Option<&str>,
        package_name: &str,
    ) -> AppResult<Vec<TargetDevice>> {
        let mut result = Vec::new();

        if !targets.device_ids.is_empty() {
            let devices = db.devices().find_by_ids(&targets.device_ids).await?;
            for device in devices {
                if let Some(aid) = app_id {
                    if device.app_id != aid {
                        return Err(AppError::BadRequest(format!(
                            "device {} does not belong to app {}",
                            device.id, aid
                        )));
                    }
                }
                result.push(device_to_target(device));
            }
        }

        if !targets.push_tokens.is_empty() {
            let platform = targets
                .platform
                .as_ref()
                .ok_or_else(|| {
                    AppError::BadRequest(
                        "targets.platform is required when using push_tokens".into(),
                    )
                })?
                .trim()
                .to_lowercase();

            if platform.is_empty() {
                return Err(AppError::BadRequest("targets.platform cannot be empty".into()));
            }

            for token in &targets.push_tokens {
                result.push(TargetDevice {
                    platform: platform.clone(),
                    push_token: token.clone(),
                    package_name: package_name.to_string(),
                    device_id: None,
                    online_token: None,
                    last_online_at: None,
                });
            }
        }

        Ok(result)
    }
}

fn merge_channels(template: &TemplateChannels, request: &TemplateChannels) -> TemplateChannels {
    TemplateChannels {
        xiaomi: request
            .xiaomi
            .clone()
            .or_else(|| template.xiaomi.clone()),
        huawei: request
            .huawei
            .clone()
            .or_else(|| template.huawei.clone()),
        oppo: request.oppo.clone().or_else(|| template.oppo.clone()),
        vivo: request.vivo.clone().or_else(|| template.vivo.clone()),
        honor: request.honor.clone().or_else(|| template.honor.clone()),
        meizu: request.meizu.clone().or_else(|| template.meizu.clone()),
    }
}

async fn default_channels_for_app(db: &Database, app_id: &str) -> AppResult<TemplateChannels> {
    let channels = db.channels().list_by_app_id(app_id).await?;
    let mut result = TemplateChannels::default();

    if let Some(code) = pick_channel_code(&channels, "xiaomi", true) {
        result.xiaomi = Some(XiaomiChannelConfig {
            channel_id: code.to_string(),
        });
    }
    if let Some(code) = pick_channel_code(&channels, "huawei", true) {
        result.huawei = Some(HuaweiCategoryConfig {
            category: code.to_uppercase(),
        });
    }
    if let Some(code) = pick_channel_code(&channels, "oppo", true) {
        result.oppo = Some(OppoChannelConfig {
            category: Some(code.to_uppercase()),
            channel_id: None,
            private_template_id: None,
        });
    }
    // 荣耀无需通道表；有推送密钥即可，category 由 HonorPushProvider 默认 NORMAL。
    if let Some(code) = pick_channel_code(&channels, "vivo", true) {
        result.vivo = Some(VivoCategoryConfig {
            category: code.to_uppercase(),
        });
    }
    if let Some(code) = pick_channel_code(&channels, "meizu", true) {
        result.meizu = Some(MeizuMsgTypeConfig {
            msg_type: code.to_uppercase(),
        });
    }

    apply_builtin_channel_defaults(&mut result);
    Ok(result)
}

async fn fill_missing_channels(
    db: &Database,
    app_id: &str,
    channels: TemplateChannels,
) -> AppResult<TemplateChannels> {
    let defaults = default_channels_for_app(db, app_id).await?;
    Ok(TemplateChannels {
        xiaomi: channels.xiaomi.or(defaults.xiaomi),
        huawei: channels.huawei.or(defaults.huawei),
        oppo: channels.oppo.or(defaults.oppo),
        vivo: channels.vivo.or(defaults.vivo),
        honor: channels.honor.or(defaults.honor),
        meizu: channels.meizu.or(defaults.meizu),
    })
}

fn pick_channel_code<'a>(
    channels: &'a [crate::models::PushChannel],
    platform: &str,
    prefer_default: bool,
) -> Option<&'a str> {
    let platform_channels: Vec<_> = channels
        .iter()
        .filter(|item| item.platform == platform)
        .collect();
    if platform_channels.is_empty() {
        return None;
    }

    if prefer_default {
        if let Some(channel) = platform_channels.iter().find(|item| item.is_default) {
            let code = channel.code.trim();
            if !code.is_empty() {
                return Some(code);
            }
        }
    }

    const PREFERRED_CODES: &[(&str, &str)] = &[
        ("huawei", "MARKETING"),
        ("vivo", "MARKETING"),
        ("oppo", "MARKETING"),
        ("meizu", "PUBLIC"),
    ];
    if let Some((_, preferred)) = PREFERRED_CODES.iter().find(|(p, _)| *p == platform) {
        if let Some(channel) = platform_channels
            .iter()
            .find(|item| item.code.eq_ignore_ascii_case(preferred))
        {
            let code = channel.code.trim();
            if !code.is_empty() {
                return Some(code);
            }
        }
    }

    platform_channels
        .first()
        .map(|item| item.code.as_str())
        .filter(|code| !code.trim().is_empty())
}

fn apply_builtin_channel_defaults(channels: &mut TemplateChannels) {
    if channels.huawei.is_none() {
        channels.huawei = Some(HuaweiCategoryConfig {
            category: "MARKETING".into(),
        });
    }
    if channels.vivo.is_none() {
        channels.vivo = Some(VivoCategoryConfig {
            category: "MARKETING".into(),
        });
    }
    if channels.meizu.is_none() {
        channels.meizu = Some(MeizuMsgTypeConfig {
            msg_type: "PUBLIC".into(),
        });
    }
    if channels.oppo.is_none() {
        channels.oppo = Some(OppoChannelConfig {
            category: Some("MARKETING".into()),
            channel_id: None,
            private_template_id: None,
        });
    }
}

fn fallback_provider_result(
    result: &crate::push::fallback::FallbackBatchResult,
) -> ProviderSendResult {
    ProviderSendResult {
        success_count: result.success_count,
        failure_count: result.failure_count,
        message_id: result.last_vendor_message_id.clone(),
        outbox_ids: vec![],
        ws_delivered: 0,
    }
}

fn device_to_target(device: Device) -> TargetDevice {
    TargetDevice {
        device_id: Some(device.id),
        platform: device.platform,
        push_token: device.push_token,
        package_name: device.package_name,
        online_token: device.online_token,
        last_online_at: device.last_online_at,
    }
}

fn online_delivery_token(target: &TargetDevice) -> String {
    target
        .online_token
        .as_ref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .unwrap_or_else(|| target.push_token.clone())
}

fn with_vendor_fallback(
    notification: &RenderedNotification,
    target: &TargetDevice,
    vendor_platform: &str,
) -> RenderedNotification {
    RenderedNotification {
        package_name: target.package_name.clone(),
        vendor_fallback: Some(PushFallbackTarget {
            platform: vendor_platform.to_string(),
            push_token: target.push_token.clone(),
        }),
        ..notification.clone()
    }
}

/// 厂商密钥已就绪且该平台推送通道已配置，才允许尝试厂商离线推送。
/// 通道未填时只走在线，不降级厂商。
fn can_use_vendor(
    platform: &str,
    configured: &[String],
    notification: &RenderedNotification,
) -> bool {
    if !is_vendor_platform(platform, configured) {
        return false;
    }
    match platform.trim().to_lowercase().as_str() {
        "xiaomi" => {
            notification.delivery_mode.is_pass_through()
                || notification.channels.xiaomi_channel_id().is_some()
        }
        "huawei" => notification.channels.huawei_category().is_some(),
        // OPPO：主路径看对应审核模板；分类只区分 IM（可不绑模板）等少数场景
        "oppo" => notification.channels.oppo_can_send(),
        "vivo" => notification.channels.vivo_category().is_some(),
        "meizu" => notification.channels.meizu_notice_msg_type().is_some(),
        // 荣耀：无独立通道配置，有密钥即可
        "honor" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_device_without_presence_still_has_online_token() {
        let target = device_to_target(Device {
            id: "1".into(),
            app_id: "app1".into(),
            package_name: "com.test".into(),
            platform: "xiaomi".into(),
            push_token: "vendor".into(),
            online_token: Some("online".into()),
            last_online_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        assert!(!target.is_recently_online(90));
        assert_eq!(online_delivery_token(&target), "online");
    }

    #[test]
    fn online_only_device_uses_push_token_when_no_online_token() {
        let target = device_to_target(Device {
            id: "1".into(),
            app_id: "app1".into(),
            package_name: "com.test".into(),
            platform: "online".into(),
            push_token: "online-token".into(),
            online_token: None,
            last_online_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        assert_eq!(online_delivery_token(&target), "online-token");
    }

    #[test]
    fn online_vendor_has_recent_presence() {
        let target = device_to_target(Device {
            id: "1".into(),
            app_id: "app1".into(),
            package_name: "com.test".into(),
            platform: "huawei".into(),
            push_token: "vendor".into(),
            online_token: Some("online".into()),
            last_online_at: Some(chrono::Utc::now() - chrono::Duration::seconds(10)),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        assert!(target.is_recently_online(90));
    }

    fn sample_notification(channels: TemplateChannels) -> RenderedNotification {
        RenderedNotification {
            title: "t".into(),
            body: "b".into(),
            payload: serde_json::Value::Null,
            click_action: Default::default(),
            package_name: "com.test".into(),
            channels,
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: HashMap::new(),
            body_variables: HashMap::new(),
        }
    }

    #[test]
    fn vendor_requires_channel_config() {
        let configured = vec!["xiaomi".into(), "online".into()];
        let no_channel = sample_notification(TemplateChannels::default());
        assert!(!can_use_vendor("xiaomi", &configured, &no_channel));

        let with_channel = sample_notification(TemplateChannels {
            xiaomi: Some(XiaomiChannelConfig {
                channel_id: "146997".into(),
            }),
            ..Default::default()
        });
        assert!(can_use_vendor("xiaomi", &configured, &with_channel));
    }

    #[test]
    fn missing_provider_cannot_use_vendor_even_with_channel() {
        let configured = vec!["online".into()];
        let with_channel = sample_notification(TemplateChannels {
            xiaomi: Some(XiaomiChannelConfig {
                channel_id: "146997".into(),
            }),
            ..Default::default()
        });
        assert!(!can_use_vendor("xiaomi", &configured, &with_channel));
    }

    #[test]
    fn xiaomi_data_mode_allows_vendor_without_channel() {
        let configured = vec!["xiaomi".into(), "online".into()];
        let mut notification = sample_notification(TemplateChannels::default());
        notification.delivery_mode = DeliveryMode::Data;
        assert!(can_use_vendor("xiaomi", &configured, &notification));
    }

    #[test]
    fn vivo_requires_secondary_category() {
        let configured = vec!["vivo".into(), "online".into()];
        let no_channel = sample_notification(TemplateChannels::default());
        assert!(!can_use_vendor("vivo", &configured, &no_channel));

        let with_channel = sample_notification(TemplateChannels {
            vivo: Some(VivoCategoryConfig {
                category: "IM".into(),
            }),
            ..Default::default()
        });
        assert!(can_use_vendor("vivo", &configured, &with_channel));
    }

    #[test]
    fn meizu_requires_public_or_private_msg_type() {
        let configured = vec!["meizu".into(), "online".into()];
        let no_channel = sample_notification(TemplateChannels::default());
        assert!(!can_use_vendor("meizu", &configured, &no_channel));

        let with_channel = sample_notification(TemplateChannels {
            meizu: Some(MeizuMsgTypeConfig {
                msg_type: "PUBLIC".into(),
            }),
            ..Default::default()
        });
        assert!(can_use_vendor("meizu", &configured, &with_channel));
    }

    #[test]
    fn huawei_requires_category_config() {
        let configured = vec!["huawei".into(), "online".into()];
        let no_channel = sample_notification(TemplateChannels::default());
        assert!(!can_use_vendor("huawei", &configured, &no_channel));

        let with_category = sample_notification(TemplateChannels {
            huawei: Some(HuaweiCategoryConfig {
                category: "MARKETING".into(),
            }),
            ..Default::default()
        });
        assert!(can_use_vendor("huawei", &configured, &with_category));
    }

    #[test]
    fn honor_works_without_channel_config() {
        let configured = vec!["honor".into(), "online".into()];
        let no_channel = sample_notification(TemplateChannels::default());
        assert!(can_use_vendor("honor", &configured, &no_channel));
    }

    #[test]
    fn builtin_defaults_fill_huawei_marketing() {
        let mut channels = TemplateChannels::default();
        apply_builtin_channel_defaults(&mut channels);
        assert_eq!(
            channels.huawei.as_ref().map(|c| c.category.as_str()),
            Some("MARKETING")
        );
    }

    #[test]
    fn oppo_prefers_template_mapping_over_category() {
        let configured = vec!["oppo".into(), "online".into()];
        let without_template = sample_notification(TemplateChannels {
            oppo: Some(OppoChannelConfig {
                category: Some("ORDER".into()),
                channel_id: Some("ORDER".into()),
                private_template_id: None,
            }),
            ..Default::default()
        });
        assert!(!can_use_vendor("oppo", &configured, &without_template));

        let template_only = sample_notification(TemplateChannels {
            oppo: Some(OppoChannelConfig {
                category: None,
                channel_id: None,
                private_template_id: Some("tpl-1".into()),
            }),
            ..Default::default()
        });
        assert!(can_use_vendor("oppo", &configured, &template_only));

        let im = sample_notification(TemplateChannels {
            oppo: Some(OppoChannelConfig {
                category: Some("IM".into()),
                channel_id: Some("IM".into()),
                private_template_id: None,
            }),
            ..Default::default()
        });
        assert!(can_use_vendor("oppo", &configured, &im));
    }
}
