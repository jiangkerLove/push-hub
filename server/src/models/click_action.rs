use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 通知点击行为。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClickActionType {
    /// 打开应用（Launcher Activity）
    #[default]
    OpenApp,
    /// 打开指定 Activity（须填全类名）
    OpenPage,
    /// 打开网页（`url` 必填）
    OpenWeb,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ClickAction {
    #[serde(default)]
    pub r#type: ClickActionType,
    /// 目标 Activity 全类名，如 `com.example.app.OrderDetailActivity`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    /// 网页地址（open_web 必填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 传给目标 Activity 的 Intent extras
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub params: HashMap<String, Value>,
}

impl ClickAction {
    pub fn validate(&self) -> Result<(), String> {
        match self.r#type {
            ClickActionType::OpenApp => Ok(()),
            ClickActionType::OpenPage => {
                let Some(activity) = self.activity_class() else {
                    return Err("open_page requires activity (fully-qualified class name)".into());
                };
                if is_fully_qualified_activity(activity) {
                    Ok(())
                } else {
                    Err(
                        "open_page activity must be a fully-qualified class name, e.g. com.example.app.OrderDetailActivity"
                            .into(),
                    )
                }
            }
            ClickActionType::OpenWeb => {
                if self.url.as_ref().is_some_and(|u| !u.trim().is_empty()) {
                    Ok(())
                } else {
                    Err("open_web requires url".into())
                }
            }
        }
    }

    /// 已校验的 Activity 全类名。
    pub fn activity_class(&self) -> Option<&str> {
        self.activity
            .as_deref()
            .map(str::trim)
            .filter(|a| !a.is_empty())
    }

    pub fn url_str(&self) -> Option<&str> {
        self.url
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty())
    }
}

/// `com.example.app.OrderDetailActivity` 这类全限定 Java 类名。
pub fn is_fully_qualified_activity(activity: &str) -> bool {
    let activity = activity.trim();
    if activity.is_empty() || activity.starts_with('.') || activity.contains('/') {
        return false;
    }
    let parts: Vec<&str> = activity.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    parts.iter().all(|part| is_java_identifier(part))
}

fn is_java_identifier(part: &str) -> bool {
    let mut chars = part.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

/// Intent component：同包全类名写成 `pkg/.Simple` / `pkg/.ui.Page`，否则 `pkg/FQCN`。
pub fn intent_component(package_name: &str, activity_fqcn: &str) -> String {
    let package_name = package_name.trim();
    let activity_fqcn = activity_fqcn.trim();
    if let Some(rest) = activity_fqcn
        .strip_prefix(package_name)
        .and_then(|r| r.strip_prefix('.'))
        .filter(|r| !r.is_empty())
    {
        format!("{package_name}/.{rest}")
    } else {
        format!("{package_name}/{activity_fqcn}")
    }
}

/// 统一 intent URI：component + CLEAR_TOP + 基本类型 extras。
pub fn build_intent_uri(
    package_name: &str,
    activity_fqcn: &str,
    params: &HashMap<String, Value>,
) -> String {
    let component = intent_component(package_name, activity_fqcn);
    let mut uri = format!("intent:#Intent;component={component};launchFlags=0x4000000;");
    append_intent_extras(&mut uri, params);
    uri.push_str("end");
    uri
}

/// Intent URI 基本类型参数（boolean / 数字 / String）。
pub fn append_intent_extras(uri: &mut String, params: &HashMap<String, Value>) {
    for (key, value) in params {
        match value {
            Value::String(text) => {
                uri.push_str(&format!("S.{key}={text};"));
            }
            Value::Bool(flag) => {
                uri.push_str(&format!("B.{key}={flag};"));
            }
            Value::Number(num) => {
                if let Some(i) = num.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        uri.push_str(&format!("i.{key}={i};"));
                    } else {
                        uri.push_str(&format!("l.{key}={i};"));
                    }
                } else if let Some(f) = num.as_f64() {
                    uri.push_str(&format!("d.{key}={f};"));
                }
            }
            _ => {}
        }
    }
}

/// 页面参数对象，供 OPPO / 魅族 / vivo 等非 intent_uri 通道使用。
pub fn click_params_object(params: &HashMap<String, Value>) -> Value {
    Value::Object(params.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fqcn_validation() {
        assert!(is_fully_qualified_activity("com.example.app.OrderDetailActivity"));
        assert!(is_fully_qualified_activity("com.jiangker.push.sample.DemoTargetActivity"));
        assert!(!is_fully_qualified_activity(".DemoTargetActivity"));
        assert!(!is_fully_qualified_activity("DemoTargetActivity"));
        assert!(!is_fully_qualified_activity("com.example/.Demo"));
        assert!(!is_fully_qualified_activity(""));
    }

    #[test]
    fn open_page_requires_fqcn() {
        let action = ClickAction {
            r#type: ClickActionType::OpenPage,
            activity: Some(".DemoTargetActivity".into()),
            ..Default::default()
        };
        assert!(action.validate().is_err());

        let action = ClickAction {
            r#type: ClickActionType::OpenPage,
            activity: Some("com.jiangker.push.sample.DemoTargetActivity".into()),
            ..Default::default()
        };
        assert!(action.validate().is_ok());
    }

    #[test]
    fn intent_component_and_uri() {
        assert_eq!(
            intent_component("com.jiangker.push.sample", "com.jiangker.push.sample.DemoTargetActivity"),
            "com.jiangker.push.sample/.DemoTargetActivity"
        );
        assert_eq!(
            intent_component("com.example.app", "com.example.app.ui.OrderActivity"),
            "com.example.app/.ui.OrderActivity"
        );

        let mut params = HashMap::new();
        params.insert("order_id".into(), serde_json::json!("42"));
        params.insert("count".into(), serde_json::json!(3));
        params.insert("vip".into(), serde_json::json!(true));
        let uri = build_intent_uri(
            "com.example.app",
            "com.example.app.OrderDetailActivity",
            &params,
        );
        assert!(uri.starts_with(
            "intent:#Intent;component=com.example.app/.OrderDetailActivity;launchFlags=0x4000000;"
        ));
        assert!(uri.contains("S.order_id=42;"));
        assert!(uri.contains("i.count=3;"));
        assert!(uri.contains("B.vip=true;"));
        assert!(uri.ends_with("end"));
    }
}
