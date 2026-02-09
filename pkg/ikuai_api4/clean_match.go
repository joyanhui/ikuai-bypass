package ikuai_api4

import "strings"

// matchCleanTag 用于清理模式的标签匹配：
// 1) 保留“完整等于 cleanTag”与“包含 cleanTag”两种兼容行为
// 2) 同时匹配旧配置字段(comment)与规则/分组名字字段
func matchCleanTag(cleanTag, legacyTagName, currentTagName string) bool {
	if cleanTag == "" {
		return false
	}

	if legacyTagName == cleanTag || strings.Contains(legacyTagName, cleanTag) {
		return true
	}
	if currentTagName == cleanTag || strings.Contains(currentTagName, cleanTag) {
		return true
	}



	return false
}
