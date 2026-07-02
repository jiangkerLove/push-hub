use serde_json::json;

use crate::models::{AppInitSnippet, PushApp};

/// 客户端 manifestPlaceholders 完整键名（与各 push-* 模块 Manifest 占位符一致）
const PLACEHOLDER_SPECS: &[(&str, PlaceholderSpec)] = &[
    ("PUSH_HUB_SERVER", PlaceholderSpec::Fixed),
    ("PUSH_HUB_APP_ID", PlaceholderSpec::AppId),
    ("XIAOMI_APP_ID", PlaceholderSpec::Field(|app| &app.xiaomi_app_id)),
    ("XIAOMI_APP_KEY", PlaceholderSpec::Field(|app| &app.xiaomi_app_key)),
    (
        "XIAOMI_CHANNEL_ID",
        PlaceholderSpec::Field(|app| &app.xiaomi_channel_id),
    ),
    ("HUAWEI_APP_ID", PlaceholderSpec::Field(|app| &app.huawei_app_id)),
    ("OPPO_APP_KEY", PlaceholderSpec::Field(|app| &app.oppo_app_key)),
    (
        "OPPO_APP_SECRET",
        PlaceholderSpec::Field(|app| &app.oppo_app_secret),
    ),
    ("VIVO_APP_ID", PlaceholderSpec::Field(|app| &app.vivo_app_id)),
    ("VIVO_APP_KEY", PlaceholderSpec::Field(|app| &app.vivo_app_key)),
    ("HONOR_APP_ID", PlaceholderSpec::Field(|app| &app.honor_app_id)),
    ("MEIZU_APP_ID", PlaceholderSpec::Field(|app| &app.meizu_app_id)),
    ("MEIZU_APP_KEY", PlaceholderSpec::Field(|app| &app.meizu_app_key)),
];

enum PlaceholderSpec {
    Fixed,
    AppId,
    Field(fn(&PushApp) -> &Option<String>),
}

pub fn generate(app: &PushApp, request_origin: Option<&str>) -> AppInitSnippet {
    let server = resolve_server_base_url(app, request_origin);

    let name_prefix = to_pascal_identifier(&app.name);
    let service_class = format!("{name_prefix}PushService");
    let application_class = format!("{name_prefix}Application");

    let kotlin = format!(
        "class {application_class} : Application() {{\n    override fun onCreate() {{\n        super.onCreate()\n        PushHub.init(\n            context = this,\n            config = PushHubConfig.Builder()\n                .messageService({service_class}::class.java)\n                .build(this),\n        )\n    }}\n}}",
    );

    let values = collect_placeholder_values(app, &server);
    let push_properties = build_push_properties(app, &server, &values);
    let manifest_placeholders = build_manifest_placeholders_json(&values);

    AppInitSnippet {
        server_base_url: server.clone(),
        package_name: app.package_name.clone(),
        push_api_key: app.push_api_key.clone(),
        kotlin,
        push_properties,
        manifest_placeholders,
        manifest_placeholders_kotlin: manifest_placeholders_kotlin(&values),
    }
}

fn collect_placeholder_values(app: &PushApp, server: &str) -> Vec<(&'static str, String)> {
    PLACEHOLDER_SPECS
        .iter()
        .map(|(key, spec)| {
            let value = match spec {
                PlaceholderSpec::Fixed => server.to_string(),
                PlaceholderSpec::AppId => app.id.clone(),
                PlaceholderSpec::Field(extract) => opt_or_empty(extract(app)),
            };
            (*key, value)
        })
        .collect()
}

fn to_pascal_identifier(name: &str) -> String {
    let base = name
        .replace([' ', '-', '_'], "")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();
    if base.is_empty() {
        return "App".into();
    }
    let mut chars = base.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "App".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::to_pascal_identifier;

    #[test]
    fn pascal_identifier_capitalizes_first_letter() {
        assert_eq!(to_pascal_identifier("demo"), "Demo");
        assert_eq!(to_pascal_identifier("Demo"), "Demo");
        assert_eq!(to_pascal_identifier("my-app"), "Myapp");
        assert_eq!(to_pascal_identifier(""), "App");
    }
}

fn build_push_properties(
    app: &PushApp,
    server: &str,
    values: &[(&str, String)],
) -> String {
    let mut lines = vec![
        format!("PUSH_HUB_SERVER={server}"),
        format!("PUSH_HUB_APP_ID={}", app.id),
        String::new(),
    ];

    for (key, value) in values {
        if *key == "PUSH_HUB_SERVER" || *key == "PUSH_HUB_APP_ID" {
            continue;
        }
        if value.is_empty() {
            lines.push(format!("# {key}="));
        } else {
            lines.push(format!("{key}={value}"));
        }
    }

    lines.join("\n")
}

fn build_manifest_placeholders_json(values: &[(&str, String)]) -> serde_json::Value {
    let mut placeholders = json!({});
    for (key, value) in values {
        placeholders[*key] = json!(value);
    }
    placeholders
}

fn manifest_placeholders_kotlin(values: &[(&str, String)]) -> String {
    let mut lines = Vec::new();
    let mut last_section: Option<&str> = None;

    for (key, value) in values {
        let section = placeholder_section(key);
        if last_section != Some(section) {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.push(format!("            // {section}"));
            last_section = Some(section);
        }
        lines.push(format!(
            "            \"{key}\" to \"{}\",",
            escape_kotlin_string(value)
        ));
    }

    format!(
        "android {{\n    defaultConfig {{\n        manifestPlaceholders += mapOf(\n{}\n        )\n    }}\n}}",
        lines.join("\n")
    )
}

fn placeholder_section(key: &str) -> &'static str {
    match key {
        "PUSH_HUB_SERVER" | "PUSH_HUB_APP_ID" => "Push Hub",
        key if key.starts_with("XIAOMI_") => "小米",
        key if key.starts_with("HUAWEI_") => "华为",
        key if key.starts_with("OPPO_") => "OPPO",
        key if key.starts_with("VIVO_") => "vivo",
        key if key.starts_with("HONOR_") => "荣耀",
        key if key.starts_with("MEIZU_") => "魅族",
        _ => "其他",
    }
}

fn opt_or_empty(value: &Option<String>) -> String {
    value
        .as_ref()
        .filter(|v| !v.is_empty())
        .cloned()
        .unwrap_or_default()
}

fn escape_kotlin_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 接入指南中的 Push Hub 服务端地址：优先与管理端请求来源一致。
pub fn resolve_server_base_url(app: &PushApp, request_origin: Option<&str>) -> String {
    if let Some(origin) = request_origin.filter(|v| !v.is_empty()) {
        return origin.trim().trim_end_matches('/').to_string();
    }
    app.server_base_url
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "http://10.0.2.2:3000".into())
}

pub fn origin_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let trimmed = origin.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|v| v.to_str().ok())?
        .trim();
    if host.is_empty() {
        return None;
    }
    Some(format!("{proto}://{host}"))
}
