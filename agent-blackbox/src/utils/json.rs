use serde_json::Value;

pub fn empty_object() -> Value {
    serde_json::json!({})
}
