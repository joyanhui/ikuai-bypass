use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::{Json, State};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub struct IKuaiSimulator {
    base_url: String,
    join: tokio::task::JoinHandle<()>,
}

#[derive(Clone)]
struct AppState {
    shared: Arc<Mutex<SimulatorState>>,
}

struct SimulatorState {
    username: String,
    password: String,
    next_id: AtomicI64,
    next_session: AtomicI64,
    sessions: HashSet<String>,
    custom_isps: Vec<CustomIspRecord>,
    route_objects: Vec<RouteObjectRecord>,
    stream_domains: Vec<StreamDomainRecord>,
    stream_ipports: Vec<StreamIpPortRecord>,
}

#[derive(Debug, Deserialize)]
struct LoginReq {
    username: String,
    passwd: String,
    pass: String,
}

#[derive(Debug, Deserialize)]
struct CallReqRaw {
    func_name: String,
    action: String,
    param: Value,
}

#[derive(Debug, Clone, Serialize)]
struct CustomIspRecord {
    ipgroup: String,
    time: String,
    id: i64,
    comment: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct RouteObjectRecord {
    id: i64,
    #[serde(rename = "group_name")]
    group_name: String,
    #[serde(rename = "type")]
    kind: i64,
    #[serde(rename = "group_value")]
    group_value: Vec<HashMap<String, String>>,
    comment: String,
}

#[derive(Debug, Clone, Serialize)]
struct AddrBlock {
    custom: Value,
    object: Value,
}

#[derive(Debug, Clone, Serialize)]
struct TimeCustom {
    weekdays: String,
    start_time: String,
    end_time: String,
    #[serde(rename = "type")]
    kind: String,
    comment: String,
}

#[derive(Debug, Clone, Serialize)]
struct TimeBlock {
    custom: Vec<TimeCustom>,
    object: Value,
}

#[derive(Debug, Clone, Serialize)]
struct StreamDomainRecord {
    id: i64,
    enabled: String,
    #[serde(rename = "tagname")]
    tagname: String,
    interface: String,
    comment: String,
    #[serde(rename = "src_addr")]
    src_addr: AddrBlock,
    domain: AddrBlock,
    time: TimeBlock,
}

#[derive(Debug, Clone, Serialize)]
struct StreamIpPortRecord {
    id: i64,
    enabled: String,
    #[serde(rename = "tagname")]
    tagname: String,
    interface: String,
    nexthop: String,
    comment: String,
    #[serde(rename = "iface_band")]
    iface_band: i64,
    mode: i64,
    protocol: String,
    #[serde(rename = "type")]
    kind: i64,
    #[serde(rename = "src_addr")]
    src_addr: AddrBlock,
    #[serde(rename = "dst_addr")]
    dst_addr: AddrBlock,
    time: TimeBlock,
}

impl IKuaiSimulator {
    pub async fn start(username: &str, password: &str) -> Result<Self, String> {
        let state = AppState {
            shared: Arc::new(Mutex::new(SimulatorState::new(username, password))),
        };
        let app = Router::new()
            .route("/Action/login", post(login_handler))
            .route("/Action/call", post(call_handler))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .map_err(|e| format!("failed to bind simulator listener: {e}"))?;
        let addr = listener
            .local_addr()
            .map_err(|e| format!("failed to get simulator addr: {e}"))?;
        let join = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        Ok(Self {
            base_url: format!("http://{}", addr),
            join,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for IKuaiSimulator {
    fn drop(&mut self) {
        self.join.abort();
    }
}

impl SimulatorState {
    fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            next_id: AtomicI64::new(1),
            next_session: AtomicI64::new(1),
            sessions: HashSet::new(),
            custom_isps: Vec::new(),
            route_objects: Vec::new(),
            stream_domains: Vec::new(),
            stream_ipports: Vec::new(),
        }
    }

    fn next_row_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    fn next_session_token(&self) -> String {
        format!(
            "sim-session-{}",
            self.next_session.fetch_add(1, Ordering::SeqCst)
        )
    }
}

async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Response, Response> {
    let mut guard = state.shared.lock().map_err(|_| api_error_response("state poisoned"))?;
    let expected_md5 = ikb_core::ikuai::md5_hex(&guard.password);
    let expected_pass = base64::engine::general_purpose::STANDARD
        .encode(format!("salt_11{}", guard.password));
    if req.username != guard.username || req.passwd != expected_md5 || req.pass != expected_pass {
        return Err(api_error_response("login failed"));
    }
    let token = guard.next_session_token();
    guard.sessions.insert(token.clone());
    drop(guard);

    let mut resp = Json(json!({
        "code": 0,
        "message": "Success",
    }))
    .into_response();
    let cookie = HeaderValue::from_str(&format!("sess_key={token}; Path=/; HttpOnly"))
        .map_err(|_| api_error_response("invalid session cookie"))?;
    resp.headers_mut().insert(SET_COOKIE, cookie);
    Ok(resp)
}

async fn call_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CallReqRaw>,
) -> Result<Response, Response> {
    let mut guard = state.shared.lock().map_err(|_| api_error_response("state poisoned"))?;
    if !is_valid_session(&guard, &headers) {
        return Ok(Json(json!({
            "code": 1,
            "message": "会话已过期",
        }))
        .into_response());
    }

    let response = match (req.func_name.as_str(), req.action.as_str()) {
        (ikb_core::ikuai::FUNC_NAME_CUSTOM_ISP, "show") => {
            show_response(&guard.custom_isps)
        }
        (ikb_core::ikuai::FUNC_NAME_CUSTOM_ISP, "add") => {
            let id = add_custom_isp(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_CUSTOM_ISP, "edit") => {
            let id = edit_custom_isp(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_CUSTOM_ISP, "del") => {
            del_custom_isp(&mut guard, &req.param)?;
            ok_response()
        }
        (ikb_core::ikuai::FUNC_NAME_ROUTE_OBJECT, "show") => {
            let kind = parse_filter_kind(&req.param)?;
            let data: Vec<RouteObjectRecord> = guard
                .route_objects
                .iter()
                .filter(|item| kind.is_none_or(|k| item.kind == k))
                .cloned()
                .collect();
            show_response(&data)
        }
        (ikb_core::ikuai::FUNC_NAME_ROUTE_OBJECT, "add") => {
            let id = add_route_object(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_ROUTE_OBJECT, "edit") => {
            let id = edit_route_object(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_ROUTE_OBJECT, "del") => {
            del_route_object(&mut guard, &req.param)?;
            ok_response()
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_DOMAIN, "show") => {
            show_response(&guard.stream_domains)
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_DOMAIN, "add") => {
            let id = add_stream_domain(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_DOMAIN, "edit") => {
            let id = edit_stream_domain(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_DOMAIN, "del") => {
            del_stream_domain(&mut guard, &req.param)?;
            ok_response()
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_IPPORT, "show") => {
            show_response(&guard.stream_ipports)
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_IPPORT, "add") => {
            let id = add_stream_ipport(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_IPPORT, "edit") => {
            let id = edit_stream_ipport(&mut guard, &req.param)?;
            rowid_response(id)
        }
        (ikb_core::ikuai::FUNC_NAME_STREAM_IPPORT, "del") => {
            del_stream_ipport(&mut guard, &req.param)?;
            ok_response()
        }
        _ => return Ok(Json(json!({ "code": 1, "message": "unsupported call" })).into_response()),
    };

    Ok(Json(response).into_response())
}

fn is_valid_session(state: &SimulatorState, headers: &HeaderMap) -> bool {
    let Some(raw_cookie) = headers.get(COOKIE).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    raw_cookie.split(';').any(|part| {
        let part = part.trim();
        let Some(token) = part.strip_prefix("sess_key=") else {
            return false;
        };
        state.sessions.contains(token)
    })
}

fn parse_filter_kind(param: &Value) -> Result<Option<i64>, Response> {
    let Some(filter) = param.get("FILTER1").and_then(Value::as_str) else {
        return Ok(None);
    };
    let mut parts = filter.split(',');
    let field = parts.next().unwrap_or_default();
    let op = parts.next().unwrap_or_default();
    let value = parts.next().unwrap_or_default();
    if field == "type" && op == "=" {
        return value
            .parse::<i64>()
            .map(Some)
            .map_err(|_| api_error_response("invalid FILTER1 type"));
    }
    Ok(None)
}

fn add_custom_isp(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = state.next_row_id();
    state.custom_isps.push(CustomIspRecord {
        id,
        name: required_string(param, "name")?,
        ipgroup: required_string(param, "ipgroup")?,
        comment: required_string(param, "comment")?,
        time: String::new(),
    });
    Ok(id)
}

fn edit_custom_isp(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = required_i64(param, "id")?;
    let item = state
        .custom_isps
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| api_error_response("custom_isp id not found"))?;
    item.name = required_string(param, "name")?;
    item.ipgroup = required_string(param, "ipgroup")?;
    item.comment = required_string(param, "comment")?;
    Ok(id)
}

fn del_custom_isp(state: &mut SimulatorState, param: &Value) -> Result<(), Response> {
    let ids = parse_id_csv(param)?;
    state.custom_isps.retain(|item| !ids.contains(&item.id));
    Ok(())
}

fn add_route_object(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = state.next_row_id();
    state.route_objects.push(RouteObjectRecord {
        id,
        group_name: required_string(param, "group_name")?,
        kind: required_i64(param, "type")?,
        group_value: parse_group_value(param)?,
        comment: optional_string(param, "comment"),
    });
    Ok(id)
}

fn edit_route_object(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = required_i64(param, "id")?;
    let item = state
        .route_objects
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| api_error_response("route_object id not found"))?;
    item.group_name = required_string(param, "group_name")?;
    item.kind = required_i64(param, "type")?;
    item.group_value = parse_group_value(param)?;
    item.comment = optional_string(param, "comment");
    Ok(id)
}

fn del_route_object(state: &mut SimulatorState, param: &Value) -> Result<(), Response> {
    let ids = parse_id_csv(param)?;
    state.route_objects.retain(|item| !ids.contains(&item.id));
    Ok(())
}

fn add_stream_domain(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = state.next_row_id();
    state.stream_domains.push(StreamDomainRecord {
        id,
        enabled: optional_string_with_default(param, "enabled", "yes"),
        tagname: required_string(param, "tagname")?,
        interface: required_string(param, "interface")?,
        comment: optional_string(param, "comment"),
        src_addr: parse_addr_block(param, "src_addr")?,
        domain: parse_addr_block(param, "domain")?,
        time: parse_time_block(param)?,
    });
    Ok(id)
}

fn edit_stream_domain(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = required_i64(param, "id")?;
    let item = state
        .stream_domains
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| api_error_response("stream_domain id not found"))?;
    item.enabled = optional_string_with_default(param, "enabled", "yes");
    item.tagname = required_string(param, "tagname")?;
    item.interface = required_string(param, "interface")?;
    item.comment = optional_string(param, "comment");
    item.src_addr = parse_addr_block(param, "src_addr")?;
    item.domain = parse_addr_block(param, "domain")?;
    item.time = parse_time_block(param)?;
    Ok(id)
}

fn del_stream_domain(state: &mut SimulatorState, param: &Value) -> Result<(), Response> {
    let ids = parse_id_csv(param)?;
    state.stream_domains.retain(|item| !ids.contains(&item.id));
    Ok(())
}

fn add_stream_ipport(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = state.next_row_id();
    state.stream_ipports.push(StreamIpPortRecord {
        id,
        enabled: optional_string_with_default(param, "enabled", "yes"),
        tagname: required_string(param, "tagname")?,
        interface: required_string(param, "interface")?,
        nexthop: required_string(param, "nexthop")?,
        comment: optional_string(param, "comment"),
        iface_band: required_i64(param, "iface_band")?,
        mode: required_i64(param, "mode")?,
        protocol: optional_string_with_default(param, "protocol", "tcp+udp"),
        kind: required_i64(param, "type")?,
        src_addr: parse_addr_block(param, "src_addr")?,
        dst_addr: parse_addr_block(param, "dst_addr")?,
        time: parse_time_block(param)?,
    });
    Ok(id)
}

fn edit_stream_ipport(state: &mut SimulatorState, param: &Value) -> Result<i64, Response> {
    let id = required_i64(param, "id")?;
    let item = state
        .stream_ipports
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| api_error_response("stream_ipport id not found"))?;
    item.enabled = optional_string_with_default(param, "enabled", "yes");
    item.tagname = required_string(param, "tagname")?;
    item.interface = required_string(param, "interface")?;
    item.nexthop = required_string(param, "nexthop")?;
    item.comment = optional_string(param, "comment");
    item.iface_band = required_i64(param, "iface_band")?;
    item.mode = required_i64(param, "mode")?;
    item.protocol = optional_string_with_default(param, "protocol", "tcp+udp");
    item.kind = required_i64(param, "type")?;
    item.src_addr = parse_addr_block(param, "src_addr")?;
    item.dst_addr = parse_addr_block(param, "dst_addr")?;
    item.time = parse_time_block(param)?;
    Ok(id)
}

fn del_stream_ipport(state: &mut SimulatorState, param: &Value) -> Result<(), Response> {
    let ids = parse_id_csv(param)?;
    state.stream_ipports.retain(|item| !ids.contains(&item.id));
    Ok(())
}

fn parse_group_value(param: &Value) -> Result<Vec<HashMap<String, String>>, Response> {
    let arr = param
        .get("group_value")
        .and_then(Value::as_array)
        .ok_or_else(|| api_error_response("missing group_value"))?;
    let mut out = Vec::new();
    for item in arr {
        let map = item
            .as_object()
            .ok_or_else(|| api_error_response("invalid group_value item"))?;
        let mut row = HashMap::new();
        for (key, value) in map {
            row.insert(key.to_string(), value.as_str().unwrap_or_default().to_string());
        }
        out.push(row);
    }
    Ok(out)
}

fn parse_addr_block(param: &Value, key: &str) -> Result<AddrBlock, Response> {
    let raw = param
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| api_error_response(&format!("missing {key}")))?;
    Ok(AddrBlock {
        custom: raw.get("custom").cloned().unwrap_or_else(|| json!([])),
        object: raw.get("object").cloned().unwrap_or_else(|| json!([])),
    })
}

fn parse_time_block(param: &Value) -> Result<TimeBlock, Response> {
    let raw = param
        .get("time")
        .and_then(Value::as_object)
        .ok_or_else(|| api_error_response("missing time"))?;
    let custom = raw
        .get("custom")
        .and_then(Value::as_array)
        .ok_or_else(|| api_error_response("missing time.custom"))?;
    let mut out = Vec::new();
    for item in custom {
        out.push(TimeCustom {
            weekdays: item
                .get("weekdays")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            start_time: item
                .get("start_time")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            end_time: item
                .get("end_time")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            kind: item
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            comment: item
                .get("comment")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }
    Ok(TimeBlock {
        custom: out,
        object: raw.get("object").cloned().unwrap_or_else(|| json!([])),
    })
}

fn parse_id_csv(param: &Value) -> Result<HashSet<i64>, Response> {
    let raw = required_string(param, "id")?;
    let mut ids = HashSet::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let id = part
            .parse::<i64>()
            .map_err(|_| api_error_response("invalid id csv"))?;
        ids.insert(id);
    }
    Ok(ids)
}

fn required_string(param: &Value, key: &str) -> Result<String, Response> {
    param
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| api_error_response(&format!("missing {key}")))
}

fn optional_string(param: &Value, key: &str) -> String {
    param.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn optional_string_with_default(param: &Value, key: &str, default: &str) -> String {
    param.get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn required_i64(param: &Value, key: &str) -> Result<i64, Response> {
    param
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| api_error_response(&format!("missing {key}")))
}

fn ok_response() -> Value {
    json!({
        "code": 0,
        "message": "Success",
    })
}

fn rowid_response(rowid: i64) -> Value {
    json!({
        "code": 0,
        "message": "Success",
        "rowid": rowid,
    })
}

fn show_response<T: Serialize>(data: &[T]) -> Value {
    json!({
        "code": 0,
        "message": "Success",
        "results": {
            "total": data.len(),
            "data": data,
        }
    })
}

fn api_error_response(message: &str) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "code": 1,
            "message": message,
        })),
    )
        .into_response()
}
