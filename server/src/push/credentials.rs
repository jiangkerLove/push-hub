use crate::models::{PushApp, ValidateAppCredentialsRequest, VendorCredentialValidation};
use crate::push::{
    HuaweiPushProvider, MeizuPushProvider, OppoPushProvider, VivoPushProvider, XiaomiPushProvider,
};
use crate::AppError;
use crate::AppResult;

pub async fn validate_app_credentials(
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> AppResult<Vec<VendorCredentialValidation>> {
    let only = request
        .platform
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase);

    let mut results = Vec::new();
    let vendors = [
        ("xiaomi", "小米"),
        ("huawei", "华为"),
        ("oppo", "OPPO"),
        ("vivo", "vivo"),
        ("honor", "荣耀"),
        ("meizu", "魅族"),
    ];

    for (platform, label) in vendors {
        if let Some(ref wanted) = only {
            if wanted != platform {
                continue;
            }
        }
        results.push(validate_vendor(platform, label, app, request).await);
    }

    Ok(results)
}

async fn validate_vendor(
    platform: &str,
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    if !is_forced_platform(request, platform) && !vendor_has_incoming(platform, request) {
        return skipped(platform, label, "未填写凭证");
    }

    match platform {
        "xiaomi" => validate_xiaomi(label, app, request).await,
        "huawei" => validate_huawei(label, app, request).await,
        "oppo" => validate_oppo(label, app, request).await,
        "vivo" => validate_vivo(label, app, request).await,
        "honor" => validate_honor(label, app, request).await,
        "meizu" => validate_meizu(label, app, request).await,
        _ => VendorCredentialValidation {
            platform: platform.into(),
            label: label.into(),
            status: "skipped".into(),
            message: "未知厂商".into(),
        },
    }
}

fn pick_value(current: &Option<String>, incoming: &Option<String>) -> Option<String> {
    incoming
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            current
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn incoming_value(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn vendor_has_incoming(platform: &str, request: &ValidateAppCredentialsRequest) -> bool {
    match platform {
        "xiaomi" => incoming_value(&request.xiaomi_app_secret).is_some(),
        "huawei" => {
            incoming_value(&request.huawei_app_id).is_some()
                || incoming_value(&request.huawei_oauth_client_id).is_some()
                || incoming_value(&request.huawei_app_secret).is_some()
        }
        "oppo" => {
            incoming_value(&request.oppo_app_key).is_some()
                || incoming_value(&request.oppo_master_secret).is_some()
        }
        "vivo" => {
            incoming_value(&request.vivo_app_id).is_some()
                || incoming_value(&request.vivo_app_key).is_some()
                || incoming_value(&request.vivo_app_secret).is_some()
        }
        "honor" => {
            incoming_value(&request.honor_app_id).is_some()
                || incoming_value(&request.honor_oauth_client_id).is_some()
                || incoming_value(&request.honor_app_secret).is_some()
        }
        "meizu" => {
            incoming_value(&request.meizu_app_id).is_some()
                || incoming_value(&request.meizu_app_secret).is_some()
        }
        _ => false,
    }
}

fn is_forced_platform(request: &ValidateAppCredentialsRequest, platform: &str) -> bool {
    request
        .platform
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some_and(|value| value.eq_ignore_ascii_case(platform))
}

fn skipped(platform: &str, label: &str, message: &str) -> VendorCredentialValidation {
    VendorCredentialValidation {
        platform: platform.into(),
        label: label.into(),
        status: "skipped".into(),
        message: message.into(),
    }
}

fn incomplete(platform: &str, label: &str, message: &str) -> VendorCredentialValidation {
    VendorCredentialValidation {
        platform: platform.into(),
        label: label.into(),
        status: "incomplete".into(),
        message: message.into(),
    }
}

fn ok(platform: &str, label: &str, message: &str) -> VendorCredentialValidation {
    VendorCredentialValidation {
        platform: platform.into(),
        label: label.into(),
        status: "ok".into(),
        message: message.into(),
    }
}

fn vendor_error_message(err: AppError) -> String {
    match err {
        AppError::Push(message) | AppError::BadRequest(message) => message,
        other => other.to_string(),
    }
}

fn failed(platform: &str, label: &str, message: String) -> VendorCredentialValidation {
    VendorCredentialValidation {
        platform: platform.into(),
        label: label.into(),
        status: "failed".into(),
        message,
    }
}

async fn validate_xiaomi(
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    let secret = match pick_value(&app.xiaomi_app_secret, &request.xiaomi_app_secret) {
        Some(value) => value,
        None => return incomplete("xiaomi", label, "请填写 App Secret"),
    };
    let package_name = pick_value(
        &Some(app.package_name.clone()),
        &request.package_name,
    )
    .unwrap_or_else(|| app.package_name.clone());
    let provider = XiaomiPushProvider::new(secret, package_name);
    match provider.validate_credentials().await {
        Ok(()) => ok("xiaomi", label, "App Secret 校验通过"),
        Err(err) => failed("xiaomi", label, vendor_error_message(err)),
    }
}

async fn validate_huawei(
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    let app_id = pick_value(&app.huawei_app_id, &request.huawei_app_id);
    let secret = pick_value(&app.huawei_app_secret, &request.huawei_app_secret);
    let (Some(app_id), Some(secret)) = (app_id, secret) else {
        return incomplete("huawei", label, "请填写 App ID 与 App Secret");
    };
    let oauth_client_id = pick_value(&app.huawei_oauth_client_id, &request.huawei_oauth_client_id)
        .unwrap_or_else(|| app_id.clone());
    let package_name = pick_value(
        &Some(app.package_name.clone()),
        &request.package_name,
    )
    .unwrap_or_else(|| app.package_name.clone());
    let provider = HuaweiPushProvider::new(app_id, oauth_client_id, secret, package_name);
    match provider.validate_credentials().await {
        Ok(()) => ok("huawei", label, "OAuth 鉴权通过"),
        Err(err) => failed("huawei", label, vendor_error_message(err)),
    }
}

async fn validate_oppo(
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    let app_key = pick_value(&app.oppo_app_key, &request.oppo_app_key);
    let master_secret = pick_value(&app.oppo_master_secret, &request.oppo_master_secret);
    let (Some(app_key), Some(master_secret)) = (app_key, master_secret) else {
        return incomplete("oppo", label, "请填写 App Key 与 Master Secret");
    };
    let provider = OppoPushProvider::new(app_key, master_secret);
    match provider.validate_credentials().await {
        Ok(()) => ok("oppo", label, "鉴权通过"),
        Err(err) => failed("oppo", label, vendor_error_message(err)),
    }
}

async fn validate_vivo(
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    let app_id = pick_value(&app.vivo_app_id, &request.vivo_app_id);
    let app_key = pick_value(&app.vivo_app_key, &request.vivo_app_key);
    let app_secret = pick_value(&app.vivo_app_secret, &request.vivo_app_secret);
    let (Some(app_id), Some(app_key), Some(app_secret)) = (app_id, app_key, app_secret) else {
        return incomplete("vivo", label, "请填写 App ID、App Key 与 App Secret");
    };
    let provider = match VivoPushProvider::new(app_id, app_key, app_secret) {
        Ok(provider) => provider,
        Err(err) => return failed("vivo", label, vendor_error_message(err)),
    };
    match provider.validate_credentials().await {
        Ok(()) => ok("vivo", label, "鉴权通过"),
        Err(err) => failed("vivo", label, vendor_error_message(err)),
    }
}

async fn validate_honor(
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    let app_id = pick_value(&app.honor_app_id, &request.honor_app_id);
    let oauth_client_id = pick_value(&app.honor_oauth_client_id, &request.honor_oauth_client_id);
    let client_secret = pick_value(&app.honor_app_secret, &request.honor_app_secret);
    if app_id.is_none() || oauth_client_id.is_none() || client_secret.is_none() {
        return incomplete("honor", label, "请填写 App ID、Client ID 与 Client Secret");
    }
    let credentials = match crate::push::vendors::honor::resolve_honor_credentials(
        app_id,
        oauth_client_id,
        client_secret,
    ) {
        Ok(credentials) => credentials,
        Err(err) => return failed("honor", label, vendor_error_message(err)),
    };
    if credentials.app_id == credentials.oauth_client_id {
        return failed(
            "honor",
            label,
            "Client ID 与 App ID 相同，通常填错了：OAuth 请填控制台「Client ID」，不是「App ID」".into(),
        );
    }
    let package_name = pick_value(
        &Some(app.package_name.clone()),
        &request.package_name,
    )
    .unwrap_or_else(|| app.package_name.clone());
    let provider = crate::push::HonorPushProvider::from_credentials(credentials, package_name);
    match provider.validate_credentials().await {
        Ok(()) => ok("honor", label, "鉴权通过"),
        Err(err) => failed("honor", label, vendor_error_message(err)),
    }
}

async fn validate_meizu(
    label: &str,
    app: &PushApp,
    request: &ValidateAppCredentialsRequest,
) -> VendorCredentialValidation {
    let app_id = pick_value(&app.meizu_app_id, &request.meizu_app_id);
    let secret = pick_value(&app.meizu_app_secret, &request.meizu_app_secret);
    let (Some(app_id), Some(secret)) = (app_id, secret) else {
        return incomplete("meizu", label, "请填写 App ID 与 App Secret");
    };
    let provider = MeizuPushProvider::new(app_id, secret);
    match provider.validate_credentials().await {
        Ok(()) => ok("meizu", label, "凭证校验通过"),
        Err(err) => failed("meizu", label, vendor_error_message(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PushApp;
    use chrono::Utc;

    fn sample_app() -> PushApp {
        let now = Utc::now();
        PushApp {
            id: "app-1".into(),
            name: "demo".into(),
            package_name: "com.example.app".into(),
            ios_bundle_id: None,
            harmony_bundle_name: None,
            description: None,
            server_base_url: None,
            push_api_key: "phk_test".into(),
            xiaomi_app_id: None,
            xiaomi_app_key: None,
            xiaomi_channel_id: None,
            xiaomi_app_secret: None,
            huawei_app_id: None,
            huawei_oauth_client_id: None,
            huawei_app_secret: None,
            oppo_app_key: None,
            oppo_app_secret: None,
            oppo_master_secret: None,
            vivo_app_id: None,
            vivo_app_key: None,
            vivo_app_secret: None,
            honor_app_id: None,
            honor_oauth_client_id: None,
            honor_app_secret: None,
            meizu_app_id: None,
            meizu_app_key: None,
            meizu_app_secret: None,
            online_push_fallback_secs: 90,
            online_message_cache_secs: 3600,
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn skips_vendor_when_request_has_no_incoming_fields() {
        let app = sample_app();
        let request = ValidateAppCredentialsRequest::default();
        assert!(!vendor_has_incoming("vivo", &request));
        let result = validate_vendor("vivo", "vivo", &app, &request).await;
        assert_eq!(result.status, "skipped");
    }

    #[tokio::test]
    async fn validates_vendor_when_request_has_any_incoming_field() {
        let app = sample_app();
        let request = ValidateAppCredentialsRequest {
            vivo_app_id: Some("1001".into()),
            ..Default::default()
        };
        assert!(vendor_has_incoming("vivo", &request));
        let result = validate_vendor("vivo", "vivo", &app, &request).await;
        assert_eq!(result.status, "incomplete");
    }

    #[tokio::test]
    async fn forced_platform_validates_even_without_incoming_fields() {
        let app = sample_app();
        let request = ValidateAppCredentialsRequest {
            platform: Some("vivo".into()),
            ..Default::default()
        };
        assert!(is_forced_platform(&request, "vivo"));
        let result = validate_vendor("vivo", "vivo", &app, &request).await;
        assert_eq!(result.status, "incomplete");
    }

    #[tokio::test]
    async fn honor_requires_client_id_for_validation() {
        let app = sample_app();
        let request = ValidateAppCredentialsRequest {
            honor_app_id: Some("104436234".into()),
            honor_app_secret: Some("secret".into()),
            ..Default::default()
        };
        let result = validate_vendor("honor", "荣耀", &app, &request).await;
        assert_eq!(result.status, "incomplete");
        assert!(result.message.contains("Client ID"));
    }

    #[tokio::test]
    async fn honor_rejects_app_id_used_as_client_id() {
        let app = sample_app();
        let request = ValidateAppCredentialsRequest {
            honor_app_id: Some("104436234".into()),
            honor_oauth_client_id: Some("104436234".into()),
            honor_app_secret: Some("secret".into()),
            ..Default::default()
        };
        let result = validate_vendor("honor", "荣耀", &app, &request).await;
        assert_eq!(result.status, "failed");
        assert!(result.message.contains("Client ID"));
    }
}
