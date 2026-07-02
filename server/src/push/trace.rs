use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use crate::db::{
    Database, NewPushJob, NewPushJobEvent, NewPushJobTarget, UpdatePushJobTarget,
};
use crate::models::push_trace::{stages, statuses};
use crate::push::ProviderSendResult;
use crate::AppResult;

pub struct PushTracer {
    db: Arc<Database>,
    job_id: String,
}

impl PushTracer {
    pub async fn start(
        db: Arc<Database>,
        app_id: &str,
        template_id: &str,
        template_name: &str,
        title: &str,
        body: &str,
        total_targets: i64,
    ) -> AppResult<Self> {
        let job_id = Uuid::new_v4().to_string();
        db.push_trace()
            .create_job(NewPushJob {
                id: job_id.clone(),
                app_id: app_id.to_string(),
                template_id: template_id.to_string(),
                template_name: template_name.to_string(),
                title: title.to_string(),
                body: body.to_string(),
                total_targets,
            })
            .await?;
        let tracer = Self { db, job_id: job_id.clone() };
        tracer
            .event(None, stages::RECEIVED, statuses::OK, None, "收到推送请求", None)
            .await?;
        Ok(tracer)
    }

    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    pub async fn template_rendered(&self, title: &str, body: &str) -> AppResult<()> {
        self.event(
            None,
            stages::TEMPLATE_RENDERED,
            statuses::OK,
            None,
            "模板渲染完成",
            Some(json!({ "title": title, "body": body })),
        )
        .await
    }

    pub async fn start_target(
        &self,
        device_id: Option<&str>,
        platform: &str,
        push_token: &str,
    ) -> AppResult<String> {
        let target_id = Uuid::new_v4().to_string();
        self.db
            .push_trace()
            .create_target(NewPushJobTarget {
                id: target_id.clone(),
                job_id: self.job_id.clone(),
                device_id: device_id.map(str::to_string),
                platform: platform.to_string(),
                push_token: push_token.to_string(),
            })
            .await?;
        Ok(target_id)
    }

    pub async fn route_selected(
        &self,
        target_id: &str,
        route: &str,
        platform: &str,
        online: bool,
    ) -> AppResult<()> {
        self.db
            .push_trace()
            .update_target(
                target_id,
                UpdatePushJobTarget {
                    route_decision: route.to_string(),
                },
            )
            .await?;
        self.event(
            Some(target_id),
            stages::ROUTE_SELECTED,
            statuses::OK,
            Some(platform),
            if online {
                "设备在线，优先走在线通道"
            } else if route == "online_first" {
                "优先走在线通道，WebSocket 未送达将立即降级厂商"
            } else if route == "online_cache" {
                "无厂商通道，消息写入在线缓存等待重连"
            } else if route == "vendor_direct" {
                "无在线 token，直接走厂商通道"
            } else {
                "走厂商离线通道"
            },
            Some(json!({ "route": route, "online": online })),
        )
        .await
    }

    pub async fn record_online_result(
        &self,
        target_id: &str,
        result: &ProviderSendResult,
    ) -> AppResult<()> {
        if let Some(batch_id) = result.message_id.as_deref() {
            self.db
                .push_trace()
                .set_job_batch_id(&self.job_id, batch_id)
                .await?;
        }
        if let Some(outbox_id) = result.outbox_ids.first() {
            self.db
                .push_trace()
                .link_outbox(outbox_id, &self.job_id, target_id)
                .await?;
            self.db
                .push_trace()
                .set_target_outbox(target_id, outbox_id)
                .await?;
        }
        self.event(
            Some(target_id),
            stages::ONLINE_ENQUEUE,
            statuses::OK,
            Some("online"),
            "写入在线 Outbox",
            Some(json!({
                "batch_id": result.message_id,
                "outbox_id": result.outbox_ids.first(),
                "outbox_ids": result.outbox_ids,
            })),
        )
        .await?;
        self.event(
            Some(target_id),
            stages::ONLINE_WS,
            if result.ws_delivered > 0 {
                statuses::OK
            } else {
                statuses::PENDING
            },
            Some("online"),
            if result.ws_delivered > 0 {
                "WebSocket 实时下发成功"
            } else {
                "WebSocket 未连接，等待轮询拉取"
            },
            Some(json!({ "ws_delivered": result.ws_delivered })),
        )
        .await?;
        self.db
            .push_trace()
            .finish_target(target_id, statuses::OK, Some("online"))
            .await
    }

    pub async fn record_vendor_result(
        &self,
        target_id: &str,
        platform: &str,
        result: &ProviderSendResult,
        detail: &str,
    ) -> AppResult<()> {
        let success = result.failure_count == 0 && result.success_count > 0;
        if let Some(vendor_message_id) = result.message_id.as_deref() {
            self.db
                .push_trace()
                .set_target_vendor_message(target_id, vendor_message_id)
                .await?;
        }
        self.db.push_trace().update_target(
            target_id,
            UpdatePushJobTarget {
                route_decision: platform.to_string(),
            },
        ).await?;
        self.db.push_trace().finish_target(
            target_id,
            if success { statuses::OK } else { statuses::FAILED },
            Some(platform),
        ).await?;
        self.event(
            Some(target_id),
            stages::VENDOR_SEND,
            if success { statuses::OK } else { statuses::FAILED },
            Some(platform),
            detail,
            if success {
                result
                    .message_id
                    .as_ref()
                    .map(|id| json!({ "vendor_message_id": id }))
            } else {
                Some(json!({ "error": detail }))
            },
        )
        .await
    }

    pub async fn finish_job(&self, success: i64, failed: i64) -> AppResult<()> {
        self.db
            .push_trace()
            .finish_job(&self.job_id, success, failed)
            .await
    }

    async fn event(
        &self,
        target_id: Option<&str>,
        stage: &str,
        status: &str,
        platform: Option<&str>,
        detail: &str,
        metadata: Option<serde_json::Value>,
    ) -> AppResult<()> {
        self.db
            .push_trace()
            .add_event(NewPushJobEvent {
                id: Uuid::new_v4().to_string(),
                job_id: self.job_id.clone(),
                target_id: target_id.map(str::to_string),
                stage: stage.to_string(),
                status: status.to_string(),
                platform: platform.map(str::to_string),
                detail: detail.to_string(),
                metadata: metadata.map(|v| v.to_string()),
            })
            .await
    }
}

pub async fn record_online_ack(db: &Database, outbox_ids: &[String]) -> AppResult<()> {
    let acks = outbox_ids
        .iter()
        .map(|id| crate::models::OnlineMessageAck {
            id: id.clone(),
            displayed: true,
            reason: None,
        })
        .collect::<Vec<_>>();
    record_online_delivery(db, &acks).await
}

pub async fn record_online_delivery(
    db: &Database,
    acks: &[crate::models::OnlineMessageAck],
) -> AppResult<()> {
    for ack in acks {
        let Some(job_id) = db.push_trace().find_job_id_by_outbox(&ack.id).await? else {
            continue;
        };
        let target_id = db.push_trace().find_target_id_by_outbox(&ack.id).await?;
        db.push_trace()
            .add_event(NewPushJobEvent {
                id: Uuid::new_v4().to_string(),
                job_id: job_id.clone(),
                target_id: target_id.clone(),
                stage: stages::ONLINE_ACK.to_string(),
                status: statuses::OK.to_string(),
                platform: Some("online".into()),
                detail: "客户端已收到消息".into(),
                metadata: Some(
                    json!({
                        "outbox_id": ack.id,
                        "displayed": ack.displayed,
                        "reason": ack.reason,
                    })
                    .to_string(),
                ),
            })
            .await?;

        if ack.reason.as_deref() == Some("data_message") {
            continue;
        }

        let (status, detail) = if ack.displayed {
            (statuses::OK, "通知已在设备上展示")
        } else {
            (
                statuses::SKIPPED,
                match ack.reason.as_deref() {
                    Some("notification_permission_denied") => {
                        "消息已送达设备，因无通知权限未在系统栏展示"
                    }
                    _ => "消息已送达设备，但未展示通知",
                },
            )
        };

        db.push_trace()
            .add_event(NewPushJobEvent {
                id: Uuid::new_v4().to_string(),
                job_id,
                target_id,
                stage: stages::CLIENT_DISPLAY.to_string(),
                status: status.to_string(),
                platform: Some("online".into()),
                detail: detail.into(),
                metadata: Some(
                    json!({
                        "outbox_id": ack.id,
                        "displayed": ack.displayed,
                        "reason": ack.reason,
                    })
                    .to_string(),
                ),
            })
            .await?;
    }
    Ok(())
}

pub async fn record_vendor_fallback(
    db: &Database,
    outbox_id: &str,
    platform: &str,
    success: bool,
    detail: Option<&str>,
    vendor_message_id: Option<&str>,
) -> AppResult<()> {
    let Some(job_id) = db.push_trace().find_job_id_by_outbox(outbox_id).await? else {
        return Ok(());
    };
    let target_id = db.push_trace().find_target_id_by_outbox(outbox_id).await?;
    if success {
        if let (Some(target_id), Some(vendor_message_id)) = (target_id.as_deref(), vendor_message_id)
        {
            db.push_trace()
                .set_target_vendor_message(target_id, vendor_message_id)
                .await?;
            db.push_trace()
                .finish_target(target_id, statuses::OK, Some(platform))
                .await?;
        }
    } else if let Some(target_id) = target_id.as_deref() {
        db.push_trace()
            .finish_target(target_id, statuses::FAILED, Some(platform))
            .await?;
    }

    let detail_text = detail.unwrap_or_else(|| {
        if success {
            "WebSocket 未送达，已立即降级厂商离线推送"
        } else {
            "厂商降级推送失败"
        }
    });
    let mut metadata = json!({
        "outbox_id": outbox_id,
    });
    if let Some(vendor_message_id) = vendor_message_id {
        metadata["vendor_message_id"] = json!(vendor_message_id);
    }
    if !success {
        metadata["error"] = json!(detail_text);
    }
    db.push_trace()
        .add_event(NewPushJobEvent {
            id: Uuid::new_v4().to_string(),
            job_id,
            target_id,
            stage: stages::VENDOR_FALLBACK.to_string(),
            status: if success {
                statuses::OK.to_string()
            } else {
                statuses::FAILED.to_string()
            },
            platform: Some(platform.to_string()),
            detail: detail_text.to_string(),
            metadata: Some(metadata.to_string()),
        })
        .await
}
