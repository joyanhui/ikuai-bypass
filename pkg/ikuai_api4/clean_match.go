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

	// 1. 首先识别是否为受管规则
	// 名字以 IKB 开头 OR 备注包含已知标识
	isManaged := strings.HasPrefix(currentTagName, ikuai_common.NAME_PREFIX_IKB) ||
		strings.Contains(legacyTagName, ikuai_common.COMMENT_IKUAI_BYPASS) ||
		strings.Contains(legacyTagName, ikuai_common.NEW_COMMENT)

	// 如果不是受管规则，严禁删除任何内容
	if !isManaged {
		return false
	}

	// 2. 如果是 cleanAll 模式，直接确认
	if cleanTag == ikuai_common.CleanModeAll {
		return true
	}

	// 3. 执行特定标签匹配（仅限受管规则内部）
	// 备注匹配：支持完整匹配或包含匹配
	if legacyTagName != "" && (legacyTagName == cleanTag || strings.Contains(legacyTagName, cleanTag)) {
		return true
	}
	// 名字匹配：考虑到序号，使用包含匹配
	if currentTagName != "" && (currentTagName == cleanTag || strings.Contains(currentTagName, cleanTag)) {
		return true
	}

	return false
}
