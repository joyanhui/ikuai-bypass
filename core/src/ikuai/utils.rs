use serde_json::Value;

pub fn md5_hex(input: &str) -> String {
    let digest = md5::compute(input.as_bytes());
    format!("{:x}", digest)
}

pub fn to_string_list(v: &Value) -> Vec<String> {
    match v {
        Value::Null => Vec::new(),
        Value::Array(arr) => arr
            .iter()
            .map(|x| match x {
                Value::String(s) => s.clone(),
                _ => x.to_string().trim_matches('"').to_string(),
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        other => vec![other.to_string()],
    }
}

pub fn categorize_addrs(addrs: &[String]) -> (Vec<String>, Vec<String>) {
    let mut custom = Vec::new();
    let mut object = Vec::new();
    for raw in addrs {
        let addr = raw.trim();
        if addr.is_empty() {
            continue;
        }
        if addr.starts_with(super::types::NAME_PREFIX_IKB) {
            object.push(addr.to_string());
        } else if addr.contains('.') || addr.contains(':') || addr.contains('-') {
            custom.push(addr.to_string());
        } else {
            object.push(addr.to_string());
        }
    }
    (custom, object)
}
