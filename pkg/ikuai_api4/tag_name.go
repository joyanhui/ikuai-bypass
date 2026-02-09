package ikuai_api4

import (
	"regexp"
	"strconv"
	"strings"

	"ikuai-bypass/pkg/ikuai_common"
)

var tagNameSanitizer = regexp.MustCompile(`[^\p{Han}A-Za-z0-9]+`)

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

func buildIndexedTagName(raw string, index int) string {
	return buildTagName(raw) + strconv.Itoa(index+1)
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

	// 强制要求候选词必须包含 IKB 相关前缀，严禁匹配不带前缀的原始名称
	// candidates must contain IKB related prefixes, matching raw names without prefixes is strictly prohibited
	token := sanitizeTagName(raw)
	if token != "" {
		add(ikuai_common.NAME_PREFIX_IKB + token)
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
	for _, c := range buildTagNameCandidates(filterTagName) {
		if currentName == c || strings.Contains(currentName, c) {
			return true
		}
		if legacyComment == c || strings.Contains(legacyComment, c) {
			return true
		}
	}
	return false
}
