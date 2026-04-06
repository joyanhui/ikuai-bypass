use serde_json::Value;

pub fn md5_hex(input: &str) -> String {
    let digest = md5::compute(input.as_bytes());
    format!("{:x}", digest)
}

pub fn to_string_list(v: &Value) -> Vec<String> {
    match v {
        Value::Null => Vec::new(),
        Value::Array(arr) => arr.iter().map(value_to_string).collect(),
        Value::String(s) => vec![s.to_string()],
        other => vec![value_to_string(other)],
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.to_string(),
        // iKuai object references often embed the readable name in `gp_name`.
        // 爱快对象引用通常会把可读名称放在 `gp_name` 字段里。
        Value::Object(map) => {
            for key in ["gp_name", "name", "ip", "ipv6"] {
                if let Some(value) = map.get(key).and_then(Value::as_str) {
                    return value.to_string();
                }
            }
            v.to_string().trim_matches('"').to_string()
        }
        _ => v.to_string().trim_matches('"').to_string(),
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
