module("luci.controller.ikuai_bypass", package.seeall)

local fs = require "nixio.fs"
local http = require "luci.http"
local util = require "luci.util"

local helper_script = "/usr/libexec/ikuai-bypass-openwrt"
local json_codec = false
local encode_json_value
local load_json_codec

function index()
	if not fs.access(helper_script) then
		return
	end

	entry({"admin", "services", "ikuai-bypass"}, cbi("ikuai_bypass"), _("iKuai Bypass"), 60).dependent = true
	entry({"admin", "services", "ikuai-bypass", "status"}, call("action_status")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "latest"}, call("action_latest")).leaf = true
	entry({"admin", "services", "ikuai-bypass", "install"}, call("action_install")).leaf = true
end

local function trim(value)
	return (value or ""):gsub("^%s+", ""):gsub("%s+$", "")
end

local function to_bool(value)
	return value == "1" or value == "true"
end

local function normalize_channel(raw)
	local value = trim(raw):lower()
	if value == "pre" or value == "prerelease" or value == "preview" then
		return "prerelease"
	end
	return "stable"
end

local function json_response(status_code, payload)
	local encoded = nil
	local codec = nil

	http.status(status_code)
	http.prepare_content("application/json")

	codec = load_json_codec()
	if codec then
		encoded = codec.stringify(payload)
	end

	http.write(encoded or encode_json_value(payload))
end

local function escape_json_string(value)
	return (value:gsub('[%z\1-\31\\"]', function(ch)
		if ch == '"' then
			return '\\"'
		elseif ch == "\\" then
			return "\\\\"
		elseif ch == "\b" then
			return "\\b"
		elseif ch == "\f" then
			return "\\f"
		elseif ch == "\n" then
			return "\\n"
		elseif ch == "\r" then
			return "\\r"
		elseif ch == "\t" then
			return "\\t"
		end

		return string.format("\\u%04x", ch:byte())
	end))
end

local function is_json_array(value)
	local count = 0
	local max_index = 0

	for key in pairs(value) do
		if type(key) ~= "number" or key < 1 or key % 1 ~= 0 then
			return false, 0
		end

		if key > max_index then
			max_index = key
		end
		count = count + 1
	end

	return max_index == count, max_index
end

encode_json_value = function(value)
	local kind = type(value)

	if kind == "nil" then
		return "null"
	end

	if kind == "string" then
		return '"' .. escape_json_string(value) .. '"'
	end

	if kind == "number" then
		if value ~= value or value == math.huge or value == -math.huge then
			return "null"
		end
		return tostring(value)
	end

	if kind == "boolean" then
		return value and "true" or "false"
	end

	if kind ~= "table" then
		return "null"
	end

	local array_like, max_index = is_json_array(value)
	local out = {}

	if array_like then
		for i = 1, max_index do
			out[#out + 1] = encode_json_value(value[i])
		end
		return "[" .. table.concat(out, ",") .. "]"
	end

	for key, item in pairs(value) do
		out[#out + 1] = encode_json_value(tostring(key)) .. ":" .. encode_json_value(item)
	end

	return "{" .. table.concat(out, ",") .. "}"
end

load_json_codec = function()
	if json_codec ~= false then
		return json_codec
	end

	local ok, codec = pcall(require, "luci.jsonc")
	if ok and type(codec) == "table"
		and type(codec.parse) == "function"
		and type(codec.stringify) == "function"
	then
		json_codec = {
			parse = codec.parse,
			stringify = codec.stringify,
		}
		return json_codec
	end

	ok, codec = pcall(require, "luci.json")
	if ok and type(codec) == "table"
		and type(codec.decode) == "function"
		and type(codec.encode) == "function"
	then
		json_codec = {
			parse = codec.decode,
			stringify = codec.encode,
		}
		return json_codec
	end

	json_codec = nil
	return nil
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

local function run_helper(args)
	local command = util.shellquote(helper_script)
	for _, arg in ipairs(args or {}) do
		command = command .. " " .. util.shellquote(arg)
	end

	local handle = io.popen(command .. " 2>&1")
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

local function build_status_payload(meta)
	return {
		helper_version = meta.helper_version or "",
		binary_path = meta.binary_path or "/usr/bin/ikuai-bypass",
		binary_exists = to_bool(meta.binary_exists),
		binary_version = meta.binary_version or "",
		raw_uname = meta.raw_uname or "",
		opkg_arches = meta.opkg_arches or "",
		release_suffix = meta.release_suffix or "",
		asset_name = meta.asset_name or "",
		supported = to_bool(meta.supported),
		fetch_tool = meta.fetch_tool or "",
		unzip_available = to_bool(meta.unzip_available),
		download_ready = to_bool(meta.download_ready),
		last_tag = meta.last_tag or "",
		last_asset = meta.last_asset or "",
		last_installed_at = meta.last_installed_at or "",
		config_sample_path = meta.config_sample_path or "",
	}
end

local function inspect_status()
	local output, err = run_helper({ "inspect" })
	if not output then
		return nil, err
	end

	return build_status_payload(parse_key_value_lines(output)), nil
end

local function fetch_releases()
	local output, err = run_helper({ "fetch-releases" })
	if not output then
		return nil, err
	end

	local codec = load_json_codec()
	if not codec then
		return nil, "No compatible LuCI JSON module is available"
	end

	local decoded = codec.parse(output)
	if type(decoded) ~= "table" then
		return nil, "Failed to parse GitHub API response"
	end

	return decoded, nil
end

local function select_release(releases, channel, asset_name)
	local want_prerelease = channel == "prerelease"

	for _, release in ipairs(releases or {}) do
		if type(release) == "table" and not release.draft and release.prerelease == want_prerelease then
			for _, asset in ipairs(release.assets or {}) do
				if type(asset) == "table"
					and asset.name == asset_name
					and trim(asset.browser_download_url or "") ~= ""
				then
					return release, asset
				end
			end
		end
	end

	return nil, nil
end

local function resolve_release(channel)
	local status_meta, err = inspect_status()
	if not status_meta then
		return nil, err, nil, 502
	end

	if not status_meta.supported or trim(status_meta.asset_name) == "" then
		return nil, "Current OpenWrt architecture is not supported by published CLI assets", status_meta, 400
	end

	local releases, fetch_err = fetch_releases()
	if not releases then
		return nil, fetch_err, status_meta, 502
	end

	local release, asset = select_release(releases, channel, status_meta.asset_name)
	if not release or not asset then
		local human_channel = channel == "prerelease" and "prerelease" or "stable"
		return nil, "No " .. human_channel .. " release contains " .. status_meta.asset_name, status_meta, 404
	end

	return {
		status = status_meta,
		channel = channel,
		release = {
			tag_name = release.tag_name or "",
			name = release.name or release.tag_name or "",
			prerelease = release.prerelease == true,
			html_url = release.html_url or "",
			published_at = release.published_at or release.created_at or "",
			asset_name = asset.name or "",
			download_url = asset.browser_download_url or "",
			asset_size = asset.size or 0,
			asset_digest = asset.digest or "",
		},
	}, nil
end

function action_status()
	local status_meta, err = inspect_status()
	if not status_meta then
		return json_response(502, {
			ok = false,
			message = err,
		})
	end

	json_response(200, {
		ok = true,
		status = status_meta,
	})
end

function action_latest()
	local channel = normalize_channel(http.formvalue("channel"))
	local resolved, err, status_meta, status_code = resolve_release(channel)
	if not resolved then
		return json_response(status_code or 404, {
			ok = false,
			message = err,
			channel = channel,
			status = status_meta,
		})
	end

	json_response(200, {
		ok = true,
		channel = channel,
		status = resolved.status,
		release = resolved.release,
	})
end

function action_install()
	local channel = normalize_channel(http.formvalue("channel"))
	local resolved, err, status_meta, status_code = resolve_release(channel)
	if not resolved then
		return json_response(status_code or 404, {
			ok = false,
			message = err,
			channel = channel,
			status = status_meta,
		})
	end

	local output, run_err = run_helper({
		"install",
		resolved.release.download_url,
		resolved.release.tag_name,
		resolved.release.asset_name,
	})
	if not output then
		return json_response(502, {
			ok = false,
			message = run_err,
			channel = channel,
			release = resolved.release,
			status = resolved.status,
		})
	end

	local meta = parse_key_value_lines(output)
	if meta.status == "error" then
		return json_response(502, {
			ok = false,
			message = meta.message or "Install failed",
			channel = channel,
			release = resolved.release,
			status = resolved.status,
		})
	end

	local refreshed_status, status_err = inspect_status()
	json_response(200, {
		ok = true,
		message = meta.message or "Install completed",
		channel = channel,
		release = resolved.release,
		install = {
			binary_path = meta.binary_path or resolved.status.binary_path,
			binary_version = meta.binary_version or "",
			config_sample_path = meta.config_sample_path or "",
			installed_at = meta.installed_at or "",
		},
		status = refreshed_status or resolved.status,
		status_warning = status_err,
	})
end
