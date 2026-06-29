module("luci.controller.ikuai_bypass", package.seeall)

local fs = require "nixio.fs"
local http = require "luci.http"
local jsonc = require "luci.jsonc"

local helper_script = "/usr/libexec/ikuai-bypass-openwrt"

function index()
	if not fs.access(helper_script) then
		return
	end

	entry({"admin", "services", "ikuai-bypass"}, cbi("ikuai_bypass"), _("iKuai Bypass"), 60).dependent = true
	entry({"admin", "services", "ikuai-bypass", "status"}, call("action_status")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "latest"}, call("action_latest")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "install"}, call("action_install")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "luci_version"}, call("action_luci_version")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "luci_check"}, call("action_luci_check")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "luci_update"}, call("action_luci_update")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "task"}, call("action_task")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "service"}, call("action_service")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "log"}, call("action_log")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "config_webui"}, call("action_config_webui")).leaf = true
end

local function trim(value)
	local s = (value or ""):gsub("^%s+", ""):gsub("%s+$", "")
	return s
end

local function to_bool(value)
	return value == "1" or value == "true"
end

local function json_response(status_code, payload)
	http.status(status_code)
	http.prepare_content("application/json")
	http.write(jsonc.stringify(payload))
end

local function parse_key_value_lines(text)
	local out = {}
	for line in (text or ""):gmatch("[^\r\n]+") do
		local key, value = line:match("^([%w_]+)=(.*)$")
		if key then
			out[key] = value
		end
	end
	return out
end

local function shell_quote(value)
	return "'" .. tostring(value or ""):gsub("'", "'\\''") .. "'"
end

local TASK_DIR = "/tmp/luci-ikuai-bypass-tasks"
local TASK_TTL = 300

local function cleanup_old_tasks()
	local now = os.time()
	local handle = io.popen("ls " .. TASK_DIR .. " 2>/dev/null || true")
	if not handle then return end
	for name in handle:lines() do
		local prefix = name:match("^(%d+)")
		if prefix then
			local ts = tonumber(prefix)
			if ts and (now - ts) > TASK_TTL then
				os.execute("rm -f " .. TASK_DIR .. "/" .. shell_quote(name))
			end
		end
	end
	handle:close()
end

local function run_background(args, proxy, prerelease)
	cleanup_old_tasks()
	os.execute("mkdir -p " .. TASK_DIR)
	local task_id = tostring(os.time()) .. tostring(math.random(10000, 99999))
	local out_file = TASK_DIR .. "/" .. task_id .. ".out"
	local done_file = TASK_DIR .. "/" .. task_id .. ".done"

	local cmd = "("
	if proxy and proxy ~= "" then
		cmd = cmd .. "export IKB_PROXY=" .. shell_quote(proxy) .. "; "
	end
	if prerelease and prerelease == "1" then
		cmd = cmd .. "export IKB_PRERELEASE=1; "
	end
	cmd = cmd .. shell_quote(helper_script)
	for _, arg in ipairs(args) do
		cmd = cmd .. " " .. shell_quote(arg)
	end
	cmd = cmd .. " 2>&1; echo $? > " .. shell_quote(done_file) .. ") > " .. shell_quote(out_file) .. " &"
	os.execute(cmd)
	return task_id
end

local function run_helper(args, proxy)
	local cmd = ""
	proxy = trim(proxy or "")
	if proxy ~= "" then
		cmd = "IKB_PROXY=" .. shell_quote(proxy) .. " "
	end
	cmd = cmd .. shell_quote(helper_script)
	for _, arg in ipairs(args or {}) do
		cmd = cmd .. " " .. shell_quote(arg)
	end

	local handle = io.popen(cmd .. " 2>&1")
	if not handle then
		return nil, "Failed to execute helper script"
	end

	local output = handle:read("*a") or ""
	local ok, _, code = handle:close()
	if ok == true or code == 0 then
		return output, nil
	end

	output = trim(output)
	if output == "" then
		output = "Helper script failed"
	end
	return nil, output
end

function action_status()
	local output, err = run_helper({ "inspect" })
	if not output then
		return json_response(502, { ok = false, message = err })
	end

	local meta = parse_key_value_lines(output)
	json_response(200, {
		ok = true,
		status = {
			binary_exists = to_bool(meta.binary_exists),
			binary_version = meta.binary_version or "",
			binary_path = meta.binary_path or "/opt/ikuai-bypass/ikuai-bypass",
			arch = meta.arch or "",
			service_installed = to_bool(meta.service_installed),
			config_exists = to_bool(meta.config_exists),
			running = to_bool(meta.running),
			version = meta.version or "",
		},
	})
end

function action_install()
	local proxy = http.formvalue("proxy") or ""
	local prerelease = http.formvalue("prerelease") or ""
	local task_id = run_background({ "install" }, proxy, prerelease)
	json_response(200, { ok = true, task_id = task_id })
end

function action_latest()
	local proxy = http.formvalue("proxy") or ""
	local prerelease = http.formvalue("prerelease") or ""
	local task_id = run_background({ "latest" }, proxy, prerelease)
	json_response(200, { ok = true, task_id = task_id })
end

function action_luci_version()
	local output, err = run_helper({ "luci-version" })
	if not output then
		return json_response(502, { ok = false, message = err })
	end
	local meta = parse_key_value_lines(output)
	json_response(200, { ok = true, version = meta.version or "" })
end

function action_luci_check()
	local proxy = trim(http.formvalue("proxy") or "")

	local cmd = ""
	if proxy ~= "" then
		cmd = "export http_proxy=" .. shell_quote(proxy) .. " export https_proxy=" .. shell_quote(proxy) .. "; "
	end
	cmd = cmd .. "curl -fsSL --connect-timeout 10 --max-time 30 'https://api.github.com/repos/joyanhui/ikuai-bypass/releases/latest' 2>/dev/null"

	local handle = io.popen(cmd)
	if not handle then
		return json_response(502, { ok = false, message = "Failed to run curl" })
	end
	local raw = handle:read("*a") or ""
	handle:close()

	if raw == "" then
		return json_response(502, { ok = false, message = "GitHub API returned empty response (check proxy/network)" })
	end

	local ok, data = pcall(jsonc.parse, raw)
	if not ok or type(data) ~= "table" then
		return json_response(502, { ok = false, message = "Failed to parse GitHub API response" })
	end

	local tag = data.tag_name or ""
	if tag == "" then
		return json_response(502, { ok = false, message = "GitHub API response missing tag_name" })
	end

	json_response(200, { ok = true, latest_version = tag })
end

function action_luci_update()
	local proxy = http.formvalue("proxy") or ""
	local task_id = run_background({ "luci-update" }, proxy)
	json_response(200, { ok = true, task_id = task_id })
end

function action_task()
	local task_id = trim(http.formvalue("task_id") or "")
	if task_id == "" then
		return json_response(400, { ok = false, message = "Missing task_id" })
	end

	local out_file = TASK_DIR .. "/" .. task_id .. ".out"
	local done_file = TASK_DIR .. "/" .. task_id .. ".done"

	if not fs.access(out_file) then
		return json_response(404, { ok = false, message = "Task not found" })
	end

	local log = ""
	local f = io.open(out_file, "r")
	if f then
		log = f:read("*a") or ""
		f:close()
	end

	local done = fs.access(done_file)
	local exit_code = nil
	if done then
		local df = io.open(done_file, "r")
		if df then
			exit_code = tonumber(trim(df:read("*l") or ""))
			df:close()
		end
	end

	local meta = parse_key_value_lines(log)

	json_response(200, {
		ok = true,
		done = done,
		exit_code = exit_code,
		log = log,
		meta = meta,
	})
end

function action_service()
	local action = trim(http.formvalue("action") or "")
	local allowed = {
		start = true,
		stop = true,
		restart = true,
		enable = true,
		disable = true,
		uninstall_service = true,
		uninstall_full = true,
		uninstall_self = true,
	}
	if not allowed[action] then
		return json_response(400, { ok = false, message = "Unsupported service action" })
	end

	local args = {}
	if action == "uninstall_service" then
		args = { "uninstall", "--service-only" }
	elseif action == "uninstall_full" then
		args = { "uninstall", "--full" }
	elseif action == "uninstall_self" then
		local output, err = run_helper({ "uninstall", "--full" })
		if not output then
			return json_response(502, { ok = false, message = err })
		end
		os.execute("opkg remove luci-app-ikuai-bypass --force-removal-of-dependent-packages >/dev/null 2>&1")
		return json_response(200, { ok = true, message = "Plugin uninstalled", log = output .. "\nRemoved luci-app-ikuai-bypass package" })
	else
		args = { action }
	end
	local output, err = run_helper(args)
	if not output then
		return json_response(502, { ok = false, message = err })
	end
	json_response(200, { ok = true, message = "Action completed", log = output })
end

function action_log()
	local output, err = run_helper({ "log" })
	if not output then
		return json_response(502, { ok = false, message = err })
	end
	json_response(200, { ok = true, log = output })
end

local CONFIG_PATH = "/opt/ikuai-bypass/config.yml"

function action_config_webui()
	local toggle = trim(http.formvalue("toggle") or "")
	local enable = false
	local port = "19001"
	if fs.access(CONFIG_PATH) then
		local f = io.open(CONFIG_PATH, "r")
		if f then
			local content = f:read("*a") or ""
			f:close()
			for line in content:gmatch("[^\r\n]+") do
				local v = line:match("^  enable:%s*(.*)$")
				if v then enable = (trim(v) == "true") end
				local p = line:match('^  port:%s*"?([0-9]+)"?%s*$')
				if p then port = p end
			end
			if toggle == "1" then
				local new_val = enable and "false" or "true"
				content = content:gsub('^(  enable:)%s*%S+', '%1 ' .. new_val)
				content = content:gsub('^(  enable:)%s*$', '%1 ' .. new_val)
				local tmp = CONFIG_PATH .. ".tmp." .. tostring(math.random(10000, 99999))
				os.execute("mkdir -p /opt/ikuai-bypass")
				local fw = io.open(tmp, "w")
				if fw then
					fw:write(content)
					fw:close()
					os.execute("mv " .. shell_quote(tmp) .. " " .. shell_quote(CONFIG_PATH))
					enable = not enable
				end
			end
		end
	end
	json_response(200, { ok = true, enable = enable, port = port })
end
