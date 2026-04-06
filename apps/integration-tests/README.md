# 集成测试 / Integration Tests

本目录放置 `iKuai Bypass` 的集成测试能力，包含两套后端：

- 本地默认使用 KVM/QEMU 驱动真实爱快镜像。
- GitHub CI 使用内置的 iKuai 模拟器，不再依赖在线 KVM。

所有 smoke 用例统一通过 `bash script/dev.sh cli:dev -- ...` 进入真实 CLI 开发入口，避免测试路径和日常手工使用路径脱节。

模拟器代码目录：`apps/integration-tests/src/ikuai_simulator/`

当前 smoke 用例覆盖：

- `rule_sync_update_in_place_smoke`：验证 `once -m iip` 可以创建并原地更新核心规则，且 ID 保持稳定。
- `chunked_sync_shrink_cleanup_smoke`：验证多分片规则在缩容同步后会原地更新首分片并清理冗余分片。
- `long_tag_random_suffix_truncation_smoke`：验证超长 tag 在随机后缀与截断场景下，仍能稳定识别分片并原地更新。
- `stream_rule_ipgroup_refs_smoke`：验证 `ii` 组合模式首轮同步即可正确解析基于 `ip-group` 对象引用的域名分流与端口分流。
- `safe_before_smoke`：验证远程规则下载失败时，不会清理已有旧规则。
- `safe_before_chunk_shrink_smoke`：验证多分片规则在本应缩容时若下载失败，旧分片不会被清理。
- `clean_mode_smoke`：验证 `clean` 必须显式带 `-tag`，且只清理匹配标签的规则，不误删其他标签。
- `legacy_clean_compat_smoke`：验证 `clean` 对旧版 `IKUAI_BYPASS` 备注规则的兼容清理。
- `webui_auth_runtime_config_smoke`：验证内置 WebUI 的 BasicAuth、配置保存、任务启动/停止 API 链路。
- `webui_cron_modes_smoke`：验证 CLI `cron` / `cronAft` 模式通过内置 WebUI 暴露出的状态流转与定时行为。

## 本地前置条件

- `qemu-system-x86_64` 和 `qemu-img` 已安装。
- 当前机器支持 `/dev/kvm`。
- 已准备好爱快镜像，默认路径为 `/home/y/kvm/ikuai.qcow2`。
- 已准备好 `tap0`，并且宿主机可通过该网卡向虚机提供测试规则文件。
  推荐宿主机地址：`192.168.9.2/24`，爱快管理地址：`http://192.168.9.1`。

## 运行方式

本地默认走 `auto`：优先尝试 KVM/QEMU；如果本机缺少 `qemu-system-x86_64`、`qemu-img`、镜像或 `/dev/kvm`，并且开发者显式设置了 `IKB_TEST_IKUAI_URL`，则自动退到“连接现成爱快设备”模式。

本地默认运行：

```bash
bash apps/integration-tests/run-smoke-test.sh
```

当本机存在 `qemu-system-x86_64` 且未显式设置 `IKB_TEST_IKUAI_IMAGE` 时，脚本会优先使用仓库里的 `.github/ikuai.qcow2.7z` 解压产物 `.github/ikuai.qcow2` 作为默认镜像。

如果要显式指定模拟器后端：

```bash
IKB_TEST_BACKEND=simulator bash apps/integration-tests/run-smoke-test.sh
```

如果本机没有 QEMU，但你已经有一台可访问的爱快设备，可以显式指定它的地址：

```bash
IKB_TEST_IKUAI_URL=http://192.168.1.1 \
IKB_TEST_FIXTURE_GUEST_HOST=<你的宿主机局域网IP> \
bash apps/integration-tests/run-smoke-test.sh
```

只跑单个用例：

```bash
bash apps/integration-tests/run-smoke-test.sh safe_before_smoke
```

安装本地 `pre-commit` hook：

```bash
bash apps/integration-tests/install-git-hooks.sh
```

安装脚本会把仓库内的 `.github/githooks/pre-commit` 复制到本地 `.git/hooks/pre-commit`。

## 人工复现

仓库提供了一套可直接手工复现的最小材料：

- `apps/integration-tests/manual-cli.yml`
- `apps/integration-tests/manual-fixtures/`

启动一个简单静态文件服务后，可以直接用真实开发入口手测：

```bash
python3 -m http.server 38199 --bind 0.0.0.0 --directory apps/integration-tests/manual-fixtures
bash script/dev.sh cli:dev -- -c apps/integration-tests/manual-cli.yml -r once -m iip
```

如果要验证清理路径：

```bash
bash script/dev.sh cli:dev -- -c apps/integration-tests/manual-cli.yml -r clean --tag CliBug4
```

## 常用环境变量

- `IKB_TEST_IKUAI_IMAGE`：爱快 qcow2 基础镜像路径；未显式设置时，若本机可用 QEMU，则默认取 `.github/ikuai.qcow2`。
- `IKB_TEST_BACKEND`：集成测试后端，支持 `auto`、`kvm`、`simulator`、`external`，默认 `auto`。
- `IKB_TEST_IKUAI_URL`：爱快管理地址，默认 `http://192.168.9.1`。
- `IKB_TEST_IKUAI_USERNAME`：登录用户名，默认 `admin`。
- `IKB_TEST_IKUAI_PASSWORD`：登录密码，默认 `admin888`。
- `IKB_TEST_TAP_IF`：QEMU 第一张网卡使用的 tap 设备，默认 `tap0`。
- `IKB_TEST_FIXTURE_BIND_HOST`：宿主机测试 HTTP 服务器监听地址，默认 `0.0.0.0`。
- `IKB_TEST_FIXTURE_GUEST_HOST`：爱快访问宿主机测试 HTTP 服务器时使用的地址；KVM 默认 `192.168.9.2`，外部爱快模式下通常需要显式设成宿主机局域网 IP。
- `IKB_TEST_DEV_SCRIPT`：CLI 开发入口脚本路径，默认 `script/dev.sh`。
- `IKB_TEST_QEMU_BIN` / `IKB_TEST_QEMU_IMG_BIN`：QEMU 可执行文件路径。
- `IKB_TEST_QEMU_ACCEL`：QEMU 加速器，默认 `kvm`。
- `IKB_TEST_QEMU_MEMORY`：QEMU 内存大小，默认 `4G`。
- `IKB_TEST_QEMU_SMP`：QEMU vCPU 参数，默认 `cores=4`。
- `IKB_TEST_ARTIFACT_ROOT`：测试产物目录，默认 `apps/integration-tests/.artifacts/`。

## CI 说明

`.github/workflows/integration.yml` 会在 `pull_request` 与非 tag `push` 时运行该目录下的 smoke 测试。

GitHub CI 默认设置 `IKB_TEST_BACKEND=simulator`，直接启动 `apps/integration-tests/src/ikuai_simulator/` 中的 iKuai 模拟器，再执行 smoke 用例。

本地如果需要覆盖真实爱快行为，继续使用 KVM 路径即可；CI 和本地后端分离，避免 Hosted Runner 上的虚拟化不确定性影响 PR 结果。
