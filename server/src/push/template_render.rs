use crate::models::PushTemplate;
use crate::AppResult;

/// 简单模板渲染：支持 `{{key}}` 占位符，标题与正文使用各自变量表。
pub fn render_template(
    template: &PushTemplate,
    title_variables: &std::collections::HashMap<String, String>,
    body_variables: &std::collections::HashMap<String, String>,
) -> AppResult<(String, String)> {
    let title = render_text(&template.title, title_variables);
    let body = render_text(&template.body, body_variables);
    Ok((title, body))
}

pub fn render_text(input: &str, variables: &std::collections::HashMap<String, String>) -> String {
    let mut output = input.to_string();
    for (key, value) in variables {
        output = output.replace(&format!("{{{{{key}}}}}"), value);
    }
    output
}

/// 从模板文本中提取 `{{变量名}}` 占位符（去重、保序）。
pub fn extract_template_variables(text: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let inner = &rest[start + 2..];
        let Some(end) = inner.find("}}") else {
            break;
        };
        let key = inner[..end].trim();
        if !key.is_empty() && !vars.iter().any(|item| item == key) {
            vars.push(key.to_string());
        }
        rest = &inner[end + 2..];
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::types::Json;
    use std::collections::HashMap;

    use crate::models::{ClickAction, TemplateChannels, XiaomiChannelConfig};

    fn sample_template() -> PushTemplate {
        PushTemplate {
            id: "t1".into(),
            app_id: "app1".into(),
            name: "test".into(),
            kind: "private".into(),
            content_mode: "compose".into(),
            title: "Hi {{name}}".into(),
            body: "Order {{order_no}}".into(),
            channels: Json(TemplateChannels {
                xiaomi: Some(XiaomiChannelConfig {
                    channel_id: "146997".into(),
                }),
                ..Default::default()
            }),
            click_action: Json(ClickAction::default()),
            message_cache_days: 7,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn render_template_replaces_variables() {
        let vars = [("name".into(), "张三".into()), ("order_no".into(), "001".into())]
            .into_iter()
            .collect();
        let (title, body) = render_template(&sample_template(), &vars, &vars).unwrap();
        assert_eq!(title, "Hi 张三");
        assert_eq!(body, "Order 001");
    }

    #[test]
    fn extract_template_variables_finds_placeholders() {
        let vars = extract_template_variables("Hi {{name}}");
        assert_eq!(vars, vec!["name".to_string()]);
    }

    #[test]
    fn render_template_uses_separate_variable_maps() {
        let template = PushTemplate {
            body: "Body {{body_only}}".into(),
            ..sample_template()
        };
        let mut title_vars = HashMap::new();
        title_vars.insert("name".into(), "张三".into());
        let mut body_vars = HashMap::new();
        body_vars.insert("body_only".into(), "OK".into());
        let (title, body) = render_template(&template, &title_vars, &body_vars).unwrap();
        assert_eq!(title, "Hi 张三");
        assert_eq!(body, "Body OK");
    }
}
