package ikuai_api4

import (
	"strings"

	"ikuai-bypass/pkg/ikuai_common"
)

// matchCleanTag 用于清理模式的标签匹配：
// 1) 保留“完整等于 cleanTag”与“包含 cleanTag”两种兼容行为
// 2) 同时匹配旧配置字段(comment)与规则/分组名字字段
func matchCleanTag(cleanTag, legacyTagName, currentTagName string) bool {
	if cleanTag == "" {
		return false
	}

	// 增加前缀强制校验：除非 cleanTag 显式包含前缀，否则 currentTagName 必须以 IKB 开头
	// 为了兼容旧版备注模式，legacyTagName (备注) 匹配不受此限，但 currentTagName (名字) 必须受限
	isBypassRule := strings.HasPrefix(currentTagName, ikuai_common.NAME_PREFIX_IKB) ||
		strings.HasPrefix(currentTagName, ikuai_common.NAME_PREFIX_IKB) ||
		strings.Contains(legacyTagName, ikuai_common.COMMENT_IKUAI_BYPASS)

	if !isBypassRule && cleanTag != "cleanAll" {
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
