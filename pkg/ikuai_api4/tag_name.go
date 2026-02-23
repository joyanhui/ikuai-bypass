package ikuai_api4

import (
	"regexp"
	"strconv"
	"strings"

	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
)

var tagNameSanitizer = regexp.MustCompile(`[^\p{Han}A-Za-z0-9]+`)

// 包级 logger 用于打印截断警告 / Package-level logger for truncation warnings
var tagLogger = logger.NewLogger("TAG:名称处理")

func stripKnownPrefix(raw string) string {
	raw = strings.TrimSpace(raw)
	raw = strings.TrimPrefix(raw, ikuai_common.COMMENT_IKUAI_BYPASS+"_")
	raw = strings.TrimPrefix(raw, ikuai_common.NAME_PREFIX_IKB)
	return strings.TrimSpace(raw)
}

// sanitizeTagName 移除特殊符号，仅保留中文、英文和数字。
func sanitizeTagName(raw string) string {
	return tagNameSanitizer.ReplaceAllString(stripKnownPrefix(raw), "")
}

// buildTagName 统一构建规则/分组名字，固定前缀 IKB。
func buildTagName(raw string) string {
	token := sanitizeTagName(raw)
	if token == "" {
		return ikuai_common.NAME_PREFIX_IKB
	}
	return ikuai_common.NAME_PREFIX_IKB + token
}

const maxTagNameLength = 15 // 爱快 4.0.101 对 tagname 的长度限制 / iKuai 4.0.101 tagname length limit

func buildIndexedTagName(raw string, index int) string {
	suffix := strconv.Itoa(index + 1)
	baseName := buildTagName(raw)
	originalName := baseName + suffix
	
	// 如果总长度超过限制，截断 baseName
	// Truncate baseName if total length exceeds limit
	maxBaseLen := maxTagNameLength - len(suffix)
	if len(baseName) > maxBaseLen {
		truncatedName := baseName[:maxBaseLen] + suffix
		tagLogger.Warn("TRUNCATE:名称截断", "Tag name truncated due to iKuai 15-char limit: '%s' -> '%s' (original tag: '%s')", originalName, truncatedName, raw)
		return truncatedName
	}
	
	return originalName
}

func buildTagNameCandidates(raw string) []string {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return nil
	}
	candidateSet := map[string]struct{}{}
	add := func(v string) {
		v = strings.TrimSpace(v)
		if v == "" {
			return
		}
		candidateSet[v] = struct{}{}
	}

	// 支持逗号分隔
	for _, part := range strings.Split(raw, ",") {
		token := sanitizeTagName(part)
		if token != "" {
			add(ikuai_common.NAME_PREFIX_IKB + token)
		}
	}

	res := make([]string, 0, len(candidateSet))
	for k := range candidateSet {
		res = append(res, k)
	}
	return res
}

// matchTagNameFilter 增加“名字 + 旧配置(comment)”的兼容匹配。
func matchTagNameFilter(filterTagName, currentName, legacyComment string) bool {
	if strings.TrimSpace(filterTagName) == "" {
		return true
	}
	// 强制校验 currentName 是否以 IKB 开头，除非它是旧版备注匹配
	isManaged := strings.HasPrefix(currentName, ikuai_common.NAME_PREFIX_IKB)

	for _, c := range buildTagNameCandidates(filterTagName) {
		// 如果名字以候选词开头（处理序号），且是受管规则
		if isManaged && strings.HasPrefix(currentName, c) {
			return true
		}
		// 兼容性：包含匹配（慎用，主要用于备注）
		if legacyComment != "" && strings.Contains(legacyComment, c) {
			return true
		}
	}
	return false
}
