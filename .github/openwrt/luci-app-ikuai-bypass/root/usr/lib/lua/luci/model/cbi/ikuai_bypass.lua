local form = SimpleForm(
	"ikuai_bypass",
	"iKuai Bypass",
	"This LuCI package does not bundle any CLI binary. It discovers the latest matching GitHub release and installs the router-specific CLI archive on demand."
)

local status = form:section(SimpleSection)
status.template = "ikuai_bypass/status"

return form
