use super::types::{CLEAN_MODE_ALL, COMMENT_IKUAI_BYPASS, NAME_PREFIX_IKB, NEW_COMMENT};

pub fn is_managed(comment: &str, name: &str) -> bool {
    let name = name.trim();
    if name.starts_with(NAME_PREFIX_IKB) {
        return true;
    }
    comment.contains(NEW_COMMENT) || comment.contains(COMMENT_IKUAI_BYPASS)
}

pub fn match_clean_tag(clean_tag: &str, legacy_tag_name: &str, current_tag_name: &str) -> bool {
    let clean_tag = clean_tag.trim();
    if clean_tag.is_empty() {
        return false;
    }
    if !is_managed(legacy_tag_name, current_tag_name) {
        return false;
    }
    if clean_tag == CLEAN_MODE_ALL {
        return true;
    }
    if !legacy_tag_name.is_empty()
        && (legacy_tag_name == clean_tag || legacy_tag_name.contains(clean_tag))
    {
        return true;
    }
    if !current_tag_name.is_empty()
        && (current_tag_name == clean_tag || current_tag_name.contains(clean_tag))
    {
        return true;
    }
    false
}
