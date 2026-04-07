use md5;
use once_cell::sync::Lazy;
use regex::Regex;

use super::types::{COMMENT_IKUAI_BYPASS, NAME_PREFIX_IKB};

static TAG_SANITIZER: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"[^\p{Han}A-Za-z0-9]+"));

const MAX_TAG_NAME_LENGTH: usize = 15;

const IP_GROUP_RAND_MARKER: &str = "R";

fn strip_known_prefix(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    let legacy = format!("{}{}", COMMENT_IKUAI_BYPASS, "_");
    if s.starts_with(&legacy) {
        s = s.trim_start_matches(&legacy).trim().to_string();
    }
    if s.starts_with(NAME_PREFIX_IKB) {
        s = s.trim_start_matches(NAME_PREFIX_IKB).trim().to_string();
    }
    s
}

pub fn sanitize_tag_name(raw: &str) -> String {
    let base = strip_known_prefix(raw);
    match TAG_SANITIZER.as_ref() {
        Ok(re) => re.replace_all(&base, "").to_string(),
        Err(_) => base,
    }
}

pub fn build_tag_name(raw: &str) -> String {
    let token = sanitize_tag_name(raw);
    if token.is_empty() {
        return NAME_PREFIX_IKB.to_string();
    }
    truncate_utf8_by_bytes(
        &format!("{}{}", NAME_PREFIX_IKB, token),
        MAX_TAG_NAME_LENGTH,
    )
}

pub fn build_indexed_tag_name(raw: &str, index: i64) -> String {
    let suffix = (index + 1).to_string();
    let base = build_tag_name(raw);
    let max_base_len = MAX_TAG_NAME_LENGTH.saturating_sub(suffix.len());
    let base = truncate_utf8_by_bytes(&base, max_base_len);
    format!("{}{}", base, suffix)
}

pub fn build_indexed_tag_name_with_affix(raw: &str, affix: &str, index: i64) -> String {
    let suffix = (index + 1).to_string();
    let base = build_tag_name(raw);
    let max_base_len = MAX_TAG_NAME_LENGTH.saturating_sub(affix.len() + suffix.len());
    let base = truncate_utf8_by_bytes(&base, max_base_len);
    format!("{}{}{}", base, affix, suffix)
}

fn hash_token_letters(raw: &str, len: usize) -> String {
    let len = len.clamp(1, 6);
    let digest = md5::compute(raw.trim().as_bytes());
    let mut out = String::with_capacity(len);
    for i in 0..len {
        let b = digest.0.get(i).copied().unwrap_or(0);
        let ch = (b % 26) as u8 + b'A';
        // Avoid using marker letter to make parsing easier.
        // 避免使用 marker 字母，便于解析。
        let ch = if ch == b'R' { b'S' } else { ch };
        out.push(ch as char);
    }
    out
}

/// Build indexed tag name for ip-group / ipv6-group with a deterministic random-like suffix.
/// 为 IP 分组（IPv4/IPv6）构建带“随机后缀”的分片名称（确定性 hash token）。
pub fn build_indexed_ip_group_tag_name(raw: &str, index: i64) -> String {
    let token = hash_token_letters(raw, 2);
    let affix = format!("{}{}", IP_GROUP_RAND_MARKER, token);
    build_indexed_tag_name_with_affix(raw, &affix, index)
}

fn truncate_utf8_by_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = 0usize;
    for (i, ch) in s.char_indices() {
        let next = i + ch.len_utf8();
        if next > max_bytes {
            break;
        }
        end = next;
    }
    s[..end].to_string()
}

fn build_tag_name_candidates(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    let mut set = std::collections::BTreeSet::new();
    for part in raw.split(',') {
        let token = sanitize_tag_name(part);
        if !token.is_empty() {
            set.insert(format!("{}{}", NAME_PREFIX_IKB, token));
        }
    }
    set.into_iter().collect()
}

pub fn match_tag_name_filter(filter_tag: &str, current_name: &str, legacy_comment: &str) -> bool {
    if filter_tag.trim().is_empty() {
        return true;
    }
    let is_managed = current_name.trim().starts_with(NAME_PREFIX_IKB);
    for c in build_tag_name_candidates(filter_tag) {
        let name = current_name.trim();
        if is_managed && name.starts_with(&c) {
            return true;
        }

        // Handle auto-truncated names caused by iKuai tagname length limit.
        // 兼容爱快 tagname 长度限制导致的自动截断（避免无法识别/无法原地更新）。
        if is_managed {
            let base0 = name.trim_end_matches(|ch: char| ch.is_ascii_digit());
            let base = strip_ip_group_rand_affix(base0);
            if !base.is_empty() && c.starts_with(base) {
                return true;
            }
        }
        if !legacy_comment.is_empty() && legacy_comment.contains(&c) {
            return true;
        }
    }
    false
}

fn strip_ip_group_rand_affix(s: &str) -> &str {
    // Our deterministic random-like suffix format is: "R" + 2 uppercase letters.
    // 确定性随机后缀格式："R" + 2 位大写字母。
    let b = s.as_bytes();
    if b.len() < 3 {
        return s;
    }
    let n = b.len();
    if b[n - 3] != b'R' {
        return s;
    }
    let a = b[n - 2];
    let c = b[n - 1];
    if !(a.is_ascii_uppercase() && c.is_ascii_uppercase()) {
        return s;
    }
    // ASCII suffix => safe to slice by bytes.
    &s[..n - 3]
}

#[cfg(test)]
mod tests {
    use super::build_tag_name;

    #[test]
    fn build_tag_name_truncates_to_ikuai_limit() {
        let tag = build_tag_name("SafeChunkRoute");
        assert_eq!(tag, "IKBSafeChunkRou");
        assert_eq!(tag.len(), 15);
    }
}
