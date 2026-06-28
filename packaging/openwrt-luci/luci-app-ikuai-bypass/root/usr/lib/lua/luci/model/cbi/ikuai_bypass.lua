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
	and "自动整理区域 IP、域名等到爱快，实现旁路由自动切换、域名分流、端口分流、自定义运营商、广告屏蔽等。GitHub: https://github.com/joyanhui/ikuai-bypass/"
	or "Automatically organizes regional IPs, domains, etc. into iKuai for bypass routing, domain/port分流, custom ISP, ad blocking, etc. GitHub: https://github.com/joyanhui/ikuai-bypass/"

local form = SimpleForm("ikuai_bypass", "iKuai Bypass", description)
form.reset = false
form.submit = false

local status = form:section(SimpleSection)
status.template = "ikuai_bypass/status"

return form
