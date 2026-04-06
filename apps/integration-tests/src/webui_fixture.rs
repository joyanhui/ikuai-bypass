use std::collections::HashMap;
use std::fs::{self, File};
use std::future::Future;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::StatusCode;
use serde::Serialize;

use crate::ikuai_simulator::IKuaiSimulator;

const DEFAULT_IKUAI_USERNAME: &str = "admin";
const DEFAULT_IKUAI_PASSWORD: &str = "admin888";
const DEFAULT_WEBUI_USER: &str = "webuser";
const DEFAULT_WEBUI_PASS: &str = "webpass";
const DEFAULT_READY_TIMEOUT: Duration = Duration::from_secs(30);

pub struct WebUiFixture {
    artifact_dir: PathBuf,
    config_path: PathBuf,
    webui_base_url: String,
    webui_user: String,
    webui_pass: String,
    cli_stdout_log_path: PathBuf,
    cli_stderr_log_path: PathBuf,
    cli: Child,
    _simulator: IKuaiSimulator,
    _fixture_server: FixtureServer,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebUiFixtureManifest {
    pub artifact_dir: String,
    pub config_path: String,
    pub webui_base_url: String,
    pub webui_user: String,
    pub webui_pass: String,
    pub cli_stdout_log_path: String,
    pub cli_stderr_log_path: String,
}

#[derive(Clone)]
struct FixtureResponse {
    status: u16,
    body: Vec<u8>,
    content_type: &'static str,
}

struct FixtureServer {
    guest_host: String,
    bind_addr: SocketAddr,
    state: Arc<Mutex<HashMap<String, FixtureResponse>>>,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl WebUiFixture {
    pub async fn start(artifact_root: Option<PathBuf>) -> Result<Self, String> {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|e| format!("failed to resolve workspace root: {e}"))?;
        let artifact_root = artifact_root.unwrap_or_else(|| {
            std::env::var("IKB_TEST_ARTIFACT_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| workspace_root.join("apps/integration-tests/.artifacts"))
        });
        let artifact_dir = create_artifact_dir(&artifact_root, "webui-browser-fixture")?;
        let simulator =
            IKuaiSimulator::start(DEFAULT_IKUAI_USERNAME, DEFAULT_IKUAI_PASSWORD).await?;
        let fixture_server = FixtureServer::start("127.0.0.1", "127.0.0.1")?;

        fixture_server.set_text(
            "/webui-browser/ipv4.txt",
            concat!(
                "198.18.10.1\n",
                "198.18.10.2\n",
                "198.18.10.3\n",
                "198.18.10.4\n",
                "198.18.10.5\n",
                "198.18.10.6\n",
                "198.18.10.7\n",
                "198.18.10.8\n"
            ),
        );
        fixture_server.set_text(
            "/webui-browser/domains.txt",
            concat!(
                "alpha.example.com\n",
                "beta.example.com\n",
                "gamma.example.com\n",
                "delta.example.com\n",
                "epsilon.example.com\n",
                "zeta.example.com\n",
                "eta.example.com\n",
                "theta.example.com\n"
            ),
        );

        let webui_port = reserve_local_port()?;
        let config_text = render_test_config(
            simulator.base_url(),
            DEFAULT_IKUAI_USERNAME,
            DEFAULT_IKUAI_PASSWORD,
            &format!(
                concat!(
                    "webui:\n",
                    "  enable: true\n",
                    "  port: \"{}\"\n",
                    "  user: \"{}\"\n",
                    "  pass: \"{}\"\n",
                    "  cdn-prefix: \"https://cdn.jsdelivr.net/npm\"\n",
                    "stream-domain:\n",
                    "  - interface: wan1\n",
                    "    src-addr: 192.168.77.10-192.168.77.20\n",
                    "    src-addr-opt-ipgroup: \"\"\n",
                    "    url: \"{}\"\n",
                    "    tag: BrowserDm\n",
                    "MaxNumberOfOneRecords:\n",
                    "  Isp: 5000\n",
                    "  Ipv4: 1\n",
                    "  Ipv6: 1000\n",
                    "  Domain: 1\n"
                ),
                webui_port,
                DEFAULT_WEBUI_USER,
                DEFAULT_WEBUI_PASS,
                fixture_server.url("/webui-browser/domains.txt"),
            ),
        )
        .replacen("AddWait: 10ms\n", "AddWait: 600ms\n", 1);
        ikb_core::config::Config::load_from_yaml_str(&config_text)
            .map_err(|e| format!("generated browser fixture config is invalid: {e}"))?;

        let config_path = artifact_dir.join("webui-browser-fixture.yml");
        fs::write(&config_path, &config_text).map_err(|e| {
            format!(
                "failed to write browser fixture config '{}': {e}",
                config_path.display()
            )
        })?;

        let cli_bin = ensure_cli_binary(&workspace_root)?;
        let stdout_log_path = artifact_dir.join("webui-browser-fixture.stdout.log");
        let stderr_log_path = artifact_dir.join("webui-browser-fixture.stderr.log");
        let stdout_log = File::create(&stdout_log_path)
            .map_err(|e| format!("failed to create {}: {e}", stdout_log_path.display()))?;
        let stderr_log = File::create(&stderr_log_path)
            .map_err(|e| format!("failed to create {}: {e}", stderr_log_path.display()))?;

        let config_path_arg = config_path.to_string_lossy().to_string();
        let cli = Command::new(&cli_bin)
            .current_dir(&workspace_root)
            .args([
                "-c",
                config_path_arg.as_str(),
                "-r",
                "cronAft",
                "-m",
                "ispdomain",
            ])
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn()
            .map_err(|e| {
                format!(
                    "failed to spawn browser fixture cli '{}' with config '{}': {e}",
                    cli_bin.display(),
                    config_path.display()
                )
            })?;

        let webui_base_url = format!("http://127.0.0.1:{webui_port}");
        wait_for_webui_ready(
            &webui_base_url,
            Some((DEFAULT_WEBUI_USER, DEFAULT_WEBUI_PASS)),
            DEFAULT_READY_TIMEOUT,
        )
        .await?;

        Ok(Self {
            artifact_dir,
            config_path,
            webui_base_url,
            webui_user: DEFAULT_WEBUI_USER.to_string(),
            webui_pass: DEFAULT_WEBUI_PASS.to_string(),
            cli_stdout_log_path: stdout_log_path,
            cli_stderr_log_path: stderr_log_path,
            cli,
            _simulator: simulator,
            _fixture_server: fixture_server,
        })
    }

    pub fn manifest(&self) -> WebUiFixtureManifest {
        WebUiFixtureManifest {
            artifact_dir: self.artifact_dir.display().to_string(),
            config_path: self.config_path.display().to_string(),
            webui_base_url: self.webui_base_url.clone(),
            webui_user: self.webui_user.clone(),
            webui_pass: self.webui_pass.clone(),
            cli_stdout_log_path: self.cli_stdout_log_path.display().to_string(),
            cli_stderr_log_path: self.cli_stderr_log_path.display().to_string(),
        }
    }

    pub fn write_env_file(&self, path: &Path) -> Result<(), String> {
        let manifest = self.manifest();
        let content = [
            ("IKB_WEBUI_BASE_URL", manifest.webui_base_url.as_str()),
            ("IKB_WEBUI_USER", manifest.webui_user.as_str()),
            ("IKB_WEBUI_PASS", manifest.webui_pass.as_str()),
            ("IKB_WEBUI_CONFIG_PATH", manifest.config_path.as_str()),
            ("IKB_WEBUI_ARTIFACT_DIR", manifest.artifact_dir.as_str()),
            (
                "IKB_WEBUI_CLI_STDOUT_LOG",
                manifest.cli_stdout_log_path.as_str(),
            ),
            (
                "IKB_WEBUI_CLI_STDERR_LOG",
                manifest.cli_stderr_log_path.as_str(),
            ),
        ]
        .into_iter()
        .map(|(key, value)| format!("export {key}={}\n", shell_quote(value)))
        .collect::<String>();
        fs::write(path, content)
            .map_err(|e| format!("failed to write env file '{}': {e}", path.display()))
    }

    pub fn write_manifest_json(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(&self.manifest())
            .map_err(|e| format!("failed to encode fixture manifest: {e}"))?;
        fs::write(path, text)
            .map_err(|e| format!("failed to write fixture manifest '{}': {e}", path.display()))
    }

    pub fn try_wait_cli(&mut self) -> Result<Option<ExitStatus>, String> {
        self.cli
            .try_wait()
            .map_err(|e| format!("failed to poll browser fixture cli: {e}"))
    }
}

impl Drop for WebUiFixture {
    fn drop(&mut self) {
        let _ = self.cli.kill();
        let _ = self.cli.wait();
    }
}

impl FixtureResponse {
    fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            body: body.into(),
            content_type: "text/plain; charset=utf-8",
        }
    }

    fn not_found() -> Self {
        Self {
            status: 404,
            body: b"not found\n".to_vec(),
            content_type: "text/plain; charset=utf-8",
        }
    }
}

impl FixtureServer {
    fn start(bind_host: &str, guest_host: &str) -> Result<Self, String> {
        let listener = TcpListener::bind((bind_host, 0))
            .map_err(|e| format!("failed to bind browser fixture server on {bind_host}: {e}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("failed to set browser fixture listener nonblocking: {e}"))?;
        let bind_addr = listener
            .local_addr()
            .map_err(|e| format!("failed to read browser fixture listener addr: {e}"))?;
        let state = Arc::new(Mutex::new(HashMap::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let state_for_thread = Arc::clone(&state);
        let stop_for_thread = Arc::clone(&stop);

        let join = thread::spawn(move || {
            loop {
                if stop_for_thread.load(Ordering::SeqCst) {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let state = Arc::clone(&state_for_thread);
                        let _ = thread::Builder::new()
                            .name("ikb-browser-fixture-conn".to_string())
                            .spawn(move || {
                                let _ = handle_fixture_connection(stream, state);
                            });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            guest_host: guest_host.to_string(),
            bind_addr,
            state,
            stop,
            join: Some(join),
        })
    }

    fn set_text(&self, path: &str, body: &str) {
        self.set_response(path, FixtureResponse::ok(body.as_bytes().to_vec()));
    }

    fn url(&self, path: &str) -> String {
        let normalized = normalize_path(path);
        format!(
            "http://{}:{}{}",
            self.guest_host,
            self.bind_addr.port(),
            normalized
        )
    }

    fn set_response(&self, path: &str, response: FixtureResponse) {
        self.state
            .lock()
            .expect("browser fixture state poisoned")
            .insert(normalize_path(path), response);
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", self.bind_addr.port()));
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

pub fn render_test_config(base_url: &str, username: &str, password: &str, body: &str) -> String {
    format!(
        concat!(
            "ikuai-url: \"{}\"\n",
            "username: \"{}\"\n",
            "password: \"{}\"\n",
            "cron: \"\"\n",
            "AddErrRetryWait: 10ms\n",
            "AddWait: 10ms\n",
            "github-proxy: \"\"\n",
            "proxy:\n",
            "  mode: system\n",
            "  url: \"\"\n",
            "  user: \"\"\n",
            "  pass: \"\"\n",
            "{}"
        ),
        base_url, username, password, body
    )
}

pub fn reserve_local_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| format!("failed to reserve local port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("failed to read reserved port: {e}"))?
        .port();
    drop(listener);
    Ok(port)
}

pub async fn wait_for_condition<F, Fut>(
    label: &str,
    timeout: Duration,
    interval: Duration,
    mut check: F,
) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<bool, String>>,
{
    let started = Instant::now();
    loop {
        if check().await? {
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err(format!("timed out waiting for {label}"));
        }
        tokio::time::sleep(interval).await;
    }
}

pub async fn wait_for_webui_ready(
    base_url: &str,
    auth: Option<(&str, &str)>,
    timeout: Duration,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("failed to build reqwest client: {e}"))?;
    wait_for_condition(
        "browser webui readiness",
        timeout,
        Duration::from_millis(200),
        || {
            let client = client.clone();
            let url = format!("{}/api/runtime/status", base_url.trim_end_matches('/'));
            let auth = auth.map(|(u, p)| (u.to_string(), p.to_string()));
            async move {
                let mut req = client.get(&url);
                if let Some((user, pass)) = auth {
                    req = req.basic_auth(user, Some(pass));
                }
                let resp = match req.send().await {
                    Ok(resp) => resp,
                    Err(_) => return Ok(false),
                };
                Ok(resp.status() == StatusCode::OK)
            }
        },
    )
    .await
}

fn create_artifact_dir(root: &Path, name: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(root)
        .map_err(|e| format!("failed to create artifact root '{}': {e}", root.display()))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("failed to read system time: {e}"))?
        .as_millis();
    let sanitized = sanitize_for_path(name);
    let dir = root.join(format!("{sanitized}-{ts}"));
    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create artifact dir '{}': {e}", dir.display()))?;
    Ok(dir)
}

fn ensure_cli_binary(workspace_root: &Path) -> Result<PathBuf, String> {
    let cli_bin = std::env::var("IKB_TEST_CLI_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target/debug/ikb-cli"));
    if cli_bin.is_file() {
        return Ok(cli_bin);
    }

    let output = Command::new("cargo")
        .current_dir(workspace_root)
        .args(["build", "--locked", "-p", "ikb-cli", "--bin", "ikb-cli"])
        .output()
        .map_err(|e| format!("failed to build ikb-cli binary: {e}"))?;
    if output.status.success() && cli_bin.is_file() {
        return Ok(cli_bin);
    }
    Err(format!(
        "failed to prepare ikb-cli binary\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        return path.to_string();
    }
    format!("/{path}")
}

fn sanitize_for_path(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "fixture".to_string();
    }
    out
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '/' | ':' | '.' | '='))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn handle_fixture_connection(
    mut stream: TcpStream,
    state: Arc<Mutex<HashMap<String, FixtureResponse>>>,
) -> Result<(), String> {
    let mut request_line = String::new();
    {
        let cloned = stream
            .try_clone()
            .map_err(|e| format!("failed to clone browser fixture stream: {e}"))?;
        let mut reader = BufReader::new(cloned);
        reader
            .read_line(&mut request_line)
            .map_err(|e| format!("failed to read browser fixture request line: {e}"))?;
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("failed to read browser fixture header line: {e}"))?;
            if line == "\r\n" || line.is_empty() {
                break;
            }
        }
    }

    let path = request_line
        .split_whitespace()
        .nth(1)
        .map(ToString::to_string)
        .unwrap_or_else(|| "/".to_string());
    let response = state
        .lock()
        .map_err(|_| "browser fixture state poisoned".to_string())?
        .get(&path)
        .cloned()
        .unwrap_or_else(FixtureResponse::not_found);
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
        response.status,
        reason_phrase(response.status),
        response.body.len(),
        response.content_type
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(&response.body))
        .map_err(|e| format!("failed to write browser fixture response: {e}"))?;
    Ok(())
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}
