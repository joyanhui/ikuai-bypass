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
	and "自动将通过指定的远程配置文件把区域 IP、域名等整理到爱快，实现旁路由自动切换、域名分流、端口分流（IP 分组分流）、自定义运营商、广告屏蔽等。<br><br>1. LuCI 插件 — 用来安装和卸载 ikuaibypass CLI 的可视化界面<br>2. ikuaibypass CLI — 核心工具程序，负责实际工作"
	or "Automatically organizes regional IPs, domains, and more from a remote config into iKuai, enabling bypass routing switching, domain分流, port分流 (IP group), custom ISP, ad blocking, and more.<br><br>1. LuCI Plugin — Visual interface to install/uninstall ikuaibypass CLI<br>2. ikuaibypass CLI — Core program that does the actual work"

local form = SimpleForm("ikuai_bypass", "iKuai Bypass", description)
form.reset = false
form.submit = false

local status = form:section(SimpleSection)
status.template = "ikuai_bypass/status"

return form
