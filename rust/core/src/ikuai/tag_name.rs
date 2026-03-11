use once_cell::sync::Lazy;
use regex::Regex;

use super::types::{COMMENT_IKUAI_BYPASS, NAME_PREFIX_IKB};

static TAG_SANITIZER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[^\p{Han}A-Za-z0-9]+").expect("regex"));

const MAX_TAG_NAME_LENGTH: usize = 15;

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
    TAG_SANITIZER
        .replace_all(&strip_known_prefix(raw), "")
        .to_string()
}

pub fn build_tag_name(raw: &str) -> String {
    let token = sanitize_tag_name(raw);
    if token.is_empty() {
        return NAME_PREFIX_IKB.to_string();
    }
    format!("{}{}", NAME_PREFIX_IKB, token)
}

pub fn build_indexed_tag_name(raw: &str, index: i64) -> String {
    let suffix = (index + 1).to_string();
    let base = build_tag_name(raw);
    let max_base_len = MAX_TAG_NAME_LENGTH.saturating_sub(suffix.len());
    let base = truncate_utf8_by_bytes(&base, max_base_len);
    format!("{}{}", base, suffix)
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
        if is_managed && current_name.trim().starts_with(&c) {
            return true;
        }
        if !legacy_comment.is_empty() && legacy_comment.contains(&c) {
            return true;
        }
    }
    false
}
