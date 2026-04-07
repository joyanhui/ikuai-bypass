use super::types::{CLEAN_MODE_ALL, NAME_PREFIX_IKB, managed_comment_markers};

pub fn is_managed(comment: &str, name: &str) -> bool {
    let name = name.trim();
    if name.starts_with(NAME_PREFIX_IKB) {
        return true;
    }
    managed_comment_markers()
        .into_iter()
        .any(|marker| comment.contains(marker))
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

#[cfg(test)]
mod tests {
    use super::{is_managed, match_clean_tag};

    #[test]
    fn recognizes_all_supported_comment_markers() {
        assert!(is_managed("IkuaiBypass", ""));
        assert!(is_managed("joyanhui/ikuai-bypass-2", ""));
        assert!(is_managed("IKUAI_BYPASS_demo", ""));
    }

    #[test]
    fn clean_tag_matches_legacy_repo_comment() {
        assert!(match_clean_tag("demo", "joyanhui/ikuai-bypass-demo", ""));
    }
}
