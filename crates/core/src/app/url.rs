pub fn normalize_base_url(input: &str) -> String {
    let raw = input.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.contains("://") {
        return raw.to_string();
    }
    format!("http://{}", raw)
}

pub fn normalize_url_prefix(input: &str) -> String {
    let raw = input.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.contains("://") {
        return raw.to_string();
    }
    format!("https://{}", raw)
}
