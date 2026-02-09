package ikuai_common

// 核心常量定义
const (
	// NAME_PREFIX_IKB 是用于标识 iKuai Bypass 工具创建的规则/分组名称前缀
	// 所有的规则和分组名称都会以 IKB 开头，便于识别和管理
	NAME_PREFIX_IKB = "IKB"

	// COMMENT_IKUAI_BYPASS 是旧版本使用的备注标识符
	// 仅用于向后兼容，新版本统一使用名称前缀 IKB
	COMMENT_IKUAI_BYPASS = "IKUAI_BYPASS"
	NEW_COMMENT = "ikuai-bypass"

	CleanModeAll = "cleanAll"
)
