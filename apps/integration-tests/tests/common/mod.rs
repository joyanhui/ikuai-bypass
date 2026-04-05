#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ikb_core::config::Config;
use ikb_core::ikuai::{self, IKuaiClient};
use ikb_integration_tests::ikuai_simulator::IKuaiSimulator;

const DEFAULT_IKUAI_URL: &str = "http://192.168.9.1";
const DEFAULT_IKUAI_USERNAME: &str = "admin";
const DEFAULT_IKUAI_PASSWORD: &str = "admin888";
const DEFAULT_TAP_IF: &str = "tap0";
const DEFAULT_FIXTURE_BIND_HOST: &str = "0.0.0.0";
const DEFAULT_FIXTURE_GUEST_HOST: &str = "192.168.9.2";
const DEFAULT_QEMU_BIN: &str = "qemu-system-x86_64";
const DEFAULT_QEMU_IMG_BIN: &str = "qemu-img";
const DEFAULT_QEMU_ACCEL: &str = "kvm";
const DEFAULT_QEMU_MEMORY: &str = "4G";
const DEFAULT_QEMU_SMP: &str = "cores=4";
const DEFAULT_BOOT_TIMEOUT: Duration = Duration::from_secs(180);
const READY_POLL_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestBackend {
    Auto,
    Kvm,
    Simulator,
    External,
}

#[derive(Clone)]
pub struct TestEnv {
    backend: TestBackend,
    workspace_root: PathBuf,
    artifact_root: PathBuf,
    cli_dev_script: PathBuf,
    image_path: PathBuf,
    ikuai_url: String,
    ikuai_url_explicit: bool,
    username: String,
    password: String,
    tap_if: String,
    fixture_bind_host: String,
    fixture_guest_host: String,
    qemu_bin: String,
    qemu_img_bin: String,
    qemu_accel: String,
    qemu_memory: String,
    qemu_smp: String,
}

pub struct TestHarness {
    env: TestEnv,
    artifact_dir: PathBuf,
    ikuai_base_url: String,
    fixture: FixtureServer,
    _backend: BackendInstance,
}

enum BackendInstance {
    Kvm(VmInstance),
    Simulator(IKuaiSimulator),
    External,
}

pub struct VmInstance {
    child: Child,
    stdout_log_path: PathBuf,
    stderr_log_path: PathBuf,
    overlay_path: PathBuf,
}

#[derive(Clone)]
struct FixtureResponse {
    status: u16,
    body: Vec<u8>,
    content_type: &'static str,
}

pub struct FixtureServer {
    guest_host: String,
    bind_addr: SocketAddr,
    state: Arc<Mutex<HashMap<String, FixtureResponse>>>,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl TestEnv {
    pub fn load() -> Result<Self, String> {
        let backend = TestBackend::load()?;
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|e| format!("failed to resolve workspace root: {e}"))?;
        let artifact_root = std::env::var("IKB_TEST_ARTIFACT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("apps/integration-tests/.artifacts"));
        let cli_dev_script = std::env::var("IKB_TEST_DEV_SCRIPT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("script/dev.sh"));
        let image_path = std::env::var("IKB_TEST_IKUAI_IMAGE")
            .ok()
            .map(|value| PathBuf::from(value.trim()))
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| workspace_root.join(".github/ikuai.qcow2"));
        let ikuai_url_explicit = std::env::var("IKB_TEST_IKUAI_URL")
            .ok()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);

        if !cli_dev_script.is_file() {
            return Err(format!(
                "CLI dev script not found: {}. Set IKB_TEST_DEV_SCRIPT to a valid path.",
                cli_dev_script.display()
            ));
        }
        if backend == TestBackend::Kvm && !image_path.is_file() {
            return Err(format!(
                "iKuai image not found: {}. Set IKB_TEST_IKUAI_IMAGE to a valid qcow2 path.",
                image_path.display()
            ));
        }

        let fixture_guest_host = std::env::var("IKB_TEST_FIXTURE_GUEST_HOST")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| match backend {
                TestBackend::Auto | TestBackend::Kvm | TestBackend::External => {
                    DEFAULT_FIXTURE_GUEST_HOST.to_string()
                }
                TestBackend::Simulator => "127.0.0.1".to_string(),
            });

        Ok(Self {
            backend,
            workspace_root,
            artifact_root,
            cli_dev_script,
            image_path,
            ikuai_url: env_or("IKB_TEST_IKUAI_URL", DEFAULT_IKUAI_URL),
            ikuai_url_explicit,
            username: env_or("IKB_TEST_IKUAI_USERNAME", DEFAULT_IKUAI_USERNAME),
            password: env_or("IKB_TEST_IKUAI_PASSWORD", DEFAULT_IKUAI_PASSWORD),
            tap_if: env_or("IKB_TEST_TAP_IF", DEFAULT_TAP_IF),
            fixture_bind_host: env_or("IKB_TEST_FIXTURE_BIND_HOST", DEFAULT_FIXTURE_BIND_HOST),
            fixture_guest_host,
            qemu_bin: env_or("IKB_TEST_QEMU_BIN", DEFAULT_QEMU_BIN),
            qemu_img_bin: env_or("IKB_TEST_QEMU_IMG_BIN", DEFAULT_QEMU_IMG_BIN),
            qemu_accel: env_or("IKB_TEST_QEMU_ACCEL", DEFAULT_QEMU_ACCEL),
            qemu_memory: env_or("IKB_TEST_QEMU_MEMORY", DEFAULT_QEMU_MEMORY),
            qemu_smp: env_or("IKB_TEST_QEMU_SMP", DEFAULT_QEMU_SMP),
        })
    }
}

impl TestHarness {
    pub async fn start(test_name: &str) -> Result<Self, String> {
        let env = TestEnv::load()?;
        let artifact_dir = create_artifact_dir(&env.artifact_root, test_name)?;
        let fixture = FixtureServer::start(&env, &artifact_dir)?;
        let resolved_backend = resolve_backend(&env).await?;
        let (ikuai_base_url, backend_instance) = match resolved_backend {
            TestBackend::Kvm => {
                let vm = VmInstance::start(&env, &artifact_dir).await?;
                (env.ikuai_url.clone(), BackendInstance::Kvm(vm))
            }
            TestBackend::Simulator => {
                let simulator = IKuaiSimulator::start(&env.username, &env.password).await?;
                (
                    simulator.base_url().to_string(),
                    BackendInstance::Simulator(simulator),
                )
            }
            TestBackend::External => {
                login_api(&env).await?;
                (env.ikuai_url.clone(), BackendInstance::External)
            }
            TestBackend::Auto => {
                return Err("auto backend should be resolved before start".to_string());
            }
        };
        Ok(Self {
            env,
            artifact_dir,
            ikuai_base_url,
            fixture,
            _backend: backend_instance,
        })
    }

    pub fn fixture(&self) -> &FixtureServer {
        &self.fixture
    }

    pub fn artifact_dir(&self) -> &Path {
        &self.artifact_dir
    }

    pub fn base_url(&self) -> &str {
        &self.ikuai_base_url
    }

    pub fn username(&self) -> &str {
        &self.env.username
    }

    pub fn password(&self) -> &str {
        &self.env.password
    }

    pub fn write_config(&self, file_name: &str, raw_yaml: &str) -> Result<PathBuf, String> {
        Config::load_from_yaml_str(raw_yaml)
            .map_err(|e| format!("generated test config is invalid: {e}"))?;
        let path = self.artifact_dir.join(file_name);
        fs::write(&path, raw_yaml).map_err(|e| {
            format!(
                "failed to write test config '{}': {e}",
                path.display()
            )
        })?;
        Ok(path)
    }

    pub fn run_cli(&self, args: &[&str]) -> Result<Output, String> {
        let output = Command::new("bash")
            .current_dir(&self.env.workspace_root)
            .arg(&self.env.cli_dev_script)
            .arg("cli:dev")
            .arg("--")
            .args(args)
            .output()
            .map_err(|e| {
                format!(
                    "failed to run cli:dev '{}': {e}",
                    args.join(" ")
                )
            })?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("failed to read system time: {e}"))?
            .as_millis();
        let prefix = build_log_prefix(args);
        let stdout_path = self
            .artifact_dir
            .join(format!("{}-{}.stdout.log", prefix, stamp));
        let stderr_path = self
            .artifact_dir
            .join(format!("{}-{}.stderr.log", prefix, stamp));
        fs::write(&stdout_path, &output.stdout).map_err(|e| {
            format!(
                "failed to write CLI stdout log '{}': {e}",
                stdout_path.display()
            )
        })?;
        fs::write(&stderr_path, &output.stderr).map_err(|e| {
            format!(
                "failed to write CLI stderr log '{}': {e}",
                stderr_path.display()
            )
        })?;
        Ok(output)
    }

    pub fn run_cli_expect_success(&self, args: &[&str]) -> Result<Output, String> {
        let output = self.run_cli(args)?;
        if output.status.success() {
            return Ok(output);
        }
        Err(format!(
            "cli:dev '{}' failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }

    pub fn run_cli_expect_failure(&self, args: &[&str]) -> Result<Output, String> {
        let output = self.run_cli(args)?;
        if !output.status.success() {
            return Ok(output);
        }
        Err(format!(
            "cli:dev '{}' unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }

    pub async fn login_api(&self) -> Result<IKuaiClient, String> {
        login_api_at(&self.ikuai_base_url, &self.env.username, &self.env.password).await
    }

    pub async fn clean_tag(&self, tag: &str) -> Result<(), String> {
        let api = self.login_api().await?;
        ikuai::custom_isp::del_custom_isp_all(&api, tag)
            .await
            .map_err(|e| format!("failed to clean custom ISP '{tag}': {e}"))?;
        ikuai::stream_domain::del_stream_domain_all(&api, tag)
            .await
            .map_err(|e| format!("failed to clean stream-domain '{tag}': {e}"))?;
        ikuai::ip_group::del_ikuai_bypass_ip_group(&api, tag)
            .await
            .map_err(|e| format!("failed to clean ip-group '{tag}': {e}"))?;
        ikuai::ipv6_group::del_ikuai_bypass_ipv6_group(&api, tag)
            .await
            .map_err(|e| format!("failed to clean ipv6-group '{tag}': {e}"))?;
        ikuai::stream_ipport::del_ikuai_bypass_stream_ipport(&api, tag)
            .await
            .map_err(|e| format!("failed to clean stream-ipport '{tag}': {e}"))?;
        Ok(())
    }
}

impl TestBackend {
    fn load() -> Result<Self, String> {
        match env_or("IKB_TEST_BACKEND", "auto").to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "kvm" => Ok(Self::Kvm),
            "sim" | "simulator" => Ok(Self::Simulator),
            "external" | "existing" => Ok(Self::External),
            other => Err(format!(
                "unsupported IKB_TEST_BACKEND '{}', expected 'auto', 'kvm', 'simulator', or 'external'",
                other
            )),
        }
    }
}

async fn resolve_backend(env: &TestEnv) -> Result<TestBackend, String> {
    match env.backend {
        TestBackend::Simulator => Ok(TestBackend::Simulator),
        TestBackend::Kvm => {
            ensure_kvm_prerequisites(env)?;
            Ok(TestBackend::Kvm)
        }
        TestBackend::External => {
            login_api(env).await?;
            Ok(TestBackend::External)
        }
        TestBackend::Auto => {
            let missing = kvm_prerequisites(env);
            if missing.is_empty() {
                return Ok(TestBackend::Kvm);
            }
            if env.ikuai_url_explicit {
                login_api(env).await?;
                return Ok(TestBackend::External);
            }
            Err(format!(
                "KVM prerequisites missing: {}. Install QEMU/KVM, or set IKB_TEST_IKUAI_URL to an existing iKuai instance.",
                missing.join(", ")
            ))
        }
    }
}

fn ensure_kvm_prerequisites(env: &TestEnv) -> Result<(), String> {
    let missing = kvm_prerequisites(env);
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!("KVM prerequisites missing: {}", missing.join(", ")))
}

fn kvm_prerequisites(env: &TestEnv) -> Vec<String> {
    let mut missing = Vec::new();
    if !binary_available(&env.qemu_bin) {
        missing.push(format!("qemu binary not found ({})", env.qemu_bin));
    }
    if !binary_available(&env.qemu_img_bin) {
        missing.push(format!("qemu-img binary not found ({})", env.qemu_img_bin));
    }
    if env.qemu_accel == "kvm" && !Path::new("/dev/kvm").exists() {
        missing.push("/dev/kvm not available".to_string());
    }
    if !env.image_path.is_file() {
        missing.push(format!("image missing ({})", env.image_path.display()));
    }
    missing
}

fn binary_available(binary: &str) -> bool {
    let candidate = Path::new(binary);
    if candidate.components().count() > 1 {
        return candidate.is_file();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(binary).is_file())
}

impl VmInstance {
    async fn start(env: &TestEnv, artifact_dir: &Path) -> Result<Self, String> {
        let overlay_path = artifact_dir.join("ikuai-overlay.qcow2");
        let stdout_log_path = artifact_dir.join("qemu.stdout.log");
        let stderr_log_path = artifact_dir.join("qemu.stderr.log");

        let qemu_img_output = Command::new(&env.qemu_img_bin)
            .arg("create")
            .arg("-f")
            .arg("qcow2")
            .arg("-F")
            .arg("qcow2")
            .arg("-b")
            .arg(&env.image_path)
            .arg(&overlay_path)
            .output()
            .map_err(|e| format!("failed to spawn qemu-img: {e}"))?;
        if !qemu_img_output.status.success() {
            return Err(format!(
                "qemu-img create failed\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&qemu_img_output.stdout),
                String::from_utf8_lossy(&qemu_img_output.stderr)
            ));
        }

        let stdout_log = File::create(&stdout_log_path)
            .map_err(|e| format!("failed to create {}: {e}", stdout_log_path.display()))?;
        let stderr_log = File::create(&stderr_log_path)
            .map_err(|e| format!("failed to create {}: {e}", stderr_log_path.display()))?;

        let mut command = Command::new(&env.qemu_bin);
        command
            .arg("-M")
            .arg("q35,usb=on,acpi=on,hpet=off")
            .arg("-m")
            .arg(&env.qemu_memory)
            .arg("-smp")
            .arg(&env.qemu_smp)
            .arg("-accel")
            .arg(&env.qemu_accel)
            .arg("-drive")
            .arg(format!("file={},if=virtio", overlay_path.display()))
            .arg("-device")
            .arg("usb-tablet")
            .arg("-device")
            .arg("VGA,vgamem_mb=64")
            .arg("-monitor")
            .arg("none")
            .arg("-display")
            .arg("none")
            .arg("-nic")
            .arg(format!(
                "tap,ifname={},script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11",
                env.tap_if
            ))
            .arg("-nic")
            .arg("user,model=e1000,mac=52:54:00:22:22:22")
            .arg("-nic")
            .arg("user,model=e1000,mac=52:54:00:33:33:33")
            .arg("-nic")
            .arg("user,model=e1000,mac=52:54:00:44:44:44")
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log));

        let mut child = command
            .spawn()
            .map_err(|e| format!("failed to spawn qemu-system-x86_64: {e}"))?;

        wait_for_ikuai(env, &mut child, &stderr_log_path).await?;

        Ok(Self {
            child,
            stdout_log_path,
            stderr_log_path,
            overlay_path,
        })
    }
}

impl Drop for VmInstance {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
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

    fn with_status(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            body: body.into(),
            content_type: "text/plain; charset=utf-8",
        }
    }

    fn not_found() -> Self {
        Self::with_status(404, b"not found\n".to_vec())
    }
}

impl FixtureServer {
    fn start(env: &TestEnv, _artifact_dir: &Path) -> Result<Self, String> {
        let listener = TcpListener::bind((env.fixture_bind_host.as_str(), 0)).map_err(|e| {
            format!(
                "failed to bind fixture server on {}: {e}",
                env.fixture_bind_host
            )
        })?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("failed to set fixture listener nonblocking: {e}"))?;
        let bind_addr = listener
            .local_addr()
            .map_err(|e| format!("failed to read fixture listener addr: {e}"))?;
        let state = Arc::new(Mutex::new(HashMap::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let state_for_thread = Arc::clone(&state);
        let stop_for_thread = Arc::clone(&stop);

        let join = thread::spawn(move || loop {
            if stop_for_thread.load(Ordering::SeqCst) {
                break;
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    let state = Arc::clone(&state_for_thread);
                    let _ = thread::Builder::new()
                        .name("ikb-fixture-conn".to_string())
                        .spawn(move || {
                            let _ = handle_fixture_connection(stream, state);
                        });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        });

        Ok(Self {
            guest_host: env.fixture_guest_host.clone(),
            bind_addr,
            state,
            stop,
            join: Some(join),
        })
    }

    pub fn set_text(&self, path: &str, body: &str) {
        self.set_response(path, FixtureResponse::ok(body.as_bytes().to_vec()));
    }

    pub fn set_status(&self, path: &str, status: u16, body: &str) {
        self.set_response(path, FixtureResponse::with_status(status, body.as_bytes().to_vec()));
    }

    pub fn url(&self, path: &str) -> String {
        let normalized = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        format!(
            "http://{}:{}{}",
            self.guest_host,
            self.bind_addr.port(),
            normalized
        )
    }

    fn set_response(&self, path: &str, response: FixtureResponse) {
        let normalized = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        self.state.lock().expect("fixture state poisoned").insert(normalized, response);
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

async fn login_api(env: &TestEnv) -> Result<IKuaiClient, String> {
    login_api_at(&env.ikuai_url, &env.username, &env.password).await
}

async fn login_api_at(base_url: &str, username: &str, password: &str) -> Result<IKuaiClient, String> {
    let api = IKuaiClient::new(base_url.to_string())
        .map_err(|e| format!("failed to create IKuaiClient: {e}"))?;
    api.login(username, password)
        .await
        .map_err(|e| {
            format!(
                "failed to login to {} with username '{}': {}",
                base_url, username, e
            )
        })?;
    Ok(api)
}

pub fn csv_items(value: &str) -> Vec<String> {
    let mut items: Vec<String> = value
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty() && *item != "{}" && *item != "[]")
        .map(ToString::to_string)
        .collect();
    items.sort();
    items
}

pub fn assert_stdout_contains(output: &Output, needle: &str) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains(needle) {
        return Ok(());
    }
    Err(format!(
        "expected stdout to contain '{needle}', got:\n{}",
        stdout
    ))
}

pub fn assert_stderr_contains(output: &Output, needle: &str) -> Result<(), String> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains(needle) {
        return Ok(());
    }
    Err(format!(
        "expected stderr to contain '{needle}', got:\n{}",
        stderr
    ))
}

pub fn print_failure_hint(test_name: &str, artifact_dir: &Path) {
    println!(
        "[TEST:HINT] {} artifacts: {}",
        test_name,
        artifact_dir.display()
    );
    println!(
        "[TEST:HINT] 定位失败可查看该目录下的 *.stdout.log / *.stderr.log / qemu*.log"
    );
}

pub fn render_test_config(
    base_url: &str,
    username: &str,
    password: &str,
    body: &str,
) -> String {
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

async fn wait_for_ikuai(env: &TestEnv, child: &mut Child, stderr_log_path: &Path) -> Result<(), String> {
    let started_at = Instant::now();
    let mut last_error = match login_api(env).await {
        Ok(_) => return Ok(()),
        Err(err) => err,
    };

    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("failed to poll qemu child: {e}"))?
        {
            let tail = read_tail(stderr_log_path, 120);
            return Err(format!(
                "qemu exited before iKuai became ready: {status}\nqemu stderr tail:\n{}",
                tail
            ));
        }

        if started_at.elapsed() >= DEFAULT_BOOT_TIMEOUT {
            let tail = read_tail(stderr_log_path, 120);
            return Err(format!(
                "timed out waiting for iKuai at {}\nlast error: {}\nqemu stderr tail:\n{}",
                env.ikuai_url,
                last_error,
                tail
            ));
        }

        tokio::time::sleep(READY_POLL_INTERVAL).await;

        match login_api(env).await {
            Ok(_) => return Ok(()),
            Err(err) => last_error = err,
        }
    }
}

fn create_artifact_dir(root: &Path, test_name: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(root)
        .map_err(|e| format!("failed to create artifact root '{}': {e}", root.display()))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("failed to read system time: {e}"))?
        .as_millis();
    let name = sanitize_for_path(test_name);
    let dir = root.join(format!("{}-{}", name, ts));
    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create artifact dir '{}': {e}", dir.display()))?;
    Ok(dir)
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
        return "test".to_string();
    }
    out
}

fn build_log_prefix(args: &[&str]) -> String {
    use std::hash::{Hash, Hasher};

    let joined = args.join("_");
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    joined.hash(&mut hasher);
    let hash = hasher.finish();

    let mut prefix = sanitize_for_path(&joined);
    const MAX_PREFIX: usize = 96;
    if prefix.len() > MAX_PREFIX {
        prefix.truncate(MAX_PREFIX);
    }
    format!("{}-{:x}", prefix, hash)
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn handle_fixture_connection(
    mut stream: TcpStream,
    state: Arc<Mutex<HashMap<String, FixtureResponse>>>,
) -> Result<(), String> {
    let mut request_line = String::new();
    {
        let cloned = stream
            .try_clone()
            .map_err(|e| format!("failed to clone fixture stream: {e}"))?;
        let mut reader = BufReader::new(cloned);
        reader
            .read_line(&mut request_line)
            .map_err(|e| format!("failed to read fixture request line: {e}"))?;
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("failed to read fixture header line: {e}"))?;
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
        .map_err(|_| "fixture state poisoned".to_string())?
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
        .map_err(|e| format!("failed to write fixture response: {e}"))?;
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

fn read_tail(path: &Path, max_lines: usize) -> String {
    let data = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) => {
            return format!("failed to read '{}': {}", path.display(), err);
        }
    };
    let lines: Vec<&str> = data.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}
