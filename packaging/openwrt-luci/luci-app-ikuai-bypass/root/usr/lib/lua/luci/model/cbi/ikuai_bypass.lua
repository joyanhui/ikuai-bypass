local http = require "luci.http"

local function prefers_chinese()
	local header = (http.getenv("HTTP_ACCEPT_LANGUAGE") or ""):lower()
	return header:find("zh-cn", 1, true)
		or header:find("zh-tw", 1, true)
		or header:find("zh-hk", 1, true)
		or header:find("zh-mo", 1, true)
		or header:find("zh-hans", 1, true)
		or header:find("zh-hant", 1, true)
		or header:match("^zh")
		or header:match(",%s*zh")
end

local description = prefers_chinese()
	and "该 LuCI 包不内置 CLI 二进制，会按需匹配 GitHub 最新发布并安装适合当前路由器架构的 CLI 压缩包。"
	or "This LuCI package does not bundle any CLI binary. It discovers the latest matching GitHub release and installs the router-specific CLI archive on demand."

local form = SimpleForm("ikuai_bypass", "iKuai Bypass", description)
form.reset = false
form.submit = false

local status = form:section(SimpleSection)
status.template = "ikuai_bypass/status"

return form
