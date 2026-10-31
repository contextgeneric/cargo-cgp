//! Reducing cargo's JSON stream to what an editor reads.

/// Reduce cargo's JSON message stream to the compiler messages an editor reads, as pretty JSON:
/// for each message its level, code, text, spans (file, start line and column, primary flag,
/// label), and children (level and text). Cargo's artifact and build-finished lines, and the
/// `rendered` text (which the `.cgp.stderr` pass already pins), are left out.
pub fn reduce_json(stream: &str) -> String {
    use serde_json::{Value, json};

    let field = |value: &Value, key: &str| value.get(key).cloned().unwrap_or(Value::Null);
    let spans = |message: &Value| -> Vec<Value> {
        message
            .get("spans")
            .and_then(Value::as_array)
            .map(|spans| {
                spans
                    .iter()
                    .map(|span| {
                        json!({
                            "file": field(span, "file_name"),
                            "line": field(span, "line_start"),
                            "column": field(span, "column_start"),
                            "primary": field(span, "is_primary"),
                            "label": field(span, "label"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let messages: Vec<Value> = stream
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|record| record.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .filter_map(|record| record.get("message").cloned())
        .map(|message| {
            let children: Vec<Value> = message
                .get("children")
                .and_then(Value::as_array)
                .map(|children| {
                    children
                        .iter()
                        .map(|child| {
                            json!({
                                "level": field(child, "level"),
                                "message": field(child, "message"),
                                "spans": spans(child),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            json!({
                "level": field(&message, "level"),
                "code": message.get("code").and_then(|code| code.get("code")).cloned().unwrap_or(Value::Null),
                "message": field(&message, "message"),
                "spans": spans(&message),
                "children": children,
            })
        })
        .collect();

    serde_json::to_string_pretty(&messages).expect("serializing the reduced JSON messages")
}
