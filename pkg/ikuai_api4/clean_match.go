package ikuai_api4

import "strings"

// matchCleanTag 用于清理模式的标签匹配：
// 1) 保留“完整等于 cleanTag”与“包含 cleanTag”两种兼容行为
// 2) 同时匹配备注与规则/分组名字字段
func matchCleanTag(cleanTag, comment, name string) bool {
	if cleanTag == "" {
		return false
	}

	if comment == cleanTag || strings.Contains(comment, cleanTag) {
		return true
	}
	if name == cleanTag || strings.Contains(name, cleanTag) {
		return true
	}

	// 兼容传入裸 tag 的历史行为
	commentWithPrefix := COMMENT_IKUAI_BYPASS + "_" + cleanTag
	if comment == commentWithPrefix || strings.Contains(comment, commentWithPrefix) {
		return true
	}

	// 兼容传入 IKUAI_BYPASS_xxx 时，名字按 xxx 匹配
	if strings.HasPrefix(cleanTag, COMMENT_IKUAI_BYPASS+"_") {
		tag := cleanTag[len(COMMENT_IKUAI_BYPASS)+1:]
		if name == tag || strings.Contains(name, tag) {
			return true
		}
	}

	return false
}
