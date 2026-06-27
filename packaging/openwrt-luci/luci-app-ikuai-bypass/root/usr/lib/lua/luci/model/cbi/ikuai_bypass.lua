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
	and "该 LuCI 包不内置 CLI 或安装逻辑，安装/卸载/状态等操作均临时调用远程 install.sh；代理只保存在当前浏览器。"
	or "This LuCI package does not bundle the CLI or install logic. It delegates lifecycle actions to the remote install.sh; proxy settings are stored only in this browser."

local form = SimpleForm("ikuai_bypass", "iKuai Bypass", description)
form.reset = false
form.submit = false

local status = form:section(SimpleSection)
status.template = "ikuai_bypass/status"

return form
