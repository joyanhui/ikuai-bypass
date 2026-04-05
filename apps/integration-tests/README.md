# 集成测试 / Integration Tests

本目录放置 `iKuai Bypass` 的黑盒集成测试，目标是直接驱动真实的爱快系统镜像，验证 CLI 对核心规则同步流程的端到端行为。

所有 smoke 用例统一通过 `bash script/dev.sh cli:dev -- ...` 进入真实 CLI 开发入口，避免测试路径和日常手工使用路径脱节。

当前 smoke 用例覆盖：

- `rule_sync_update_in_place_smoke`：验证 `once -m iip` 可以创建并原地更新核心规则，且 ID 保持稳定。
- `safe_before_smoke`：验证远程规则下载失败时，不会清理已有旧规则。
- `clean_mode_smoke`：验证 `clean` 必须显式带 `-tag`，且只清理匹配标签的规则，不误删其他标签。

## 前置条件

- `qemu-system-x86_64` 和 `qemu-img` 已安装。
- 当前机器支持 `/dev/kvm`。
- 已准备好爱快镜像，默认路径为 `/home/y/kvm/ikuai.qcow2`。
- 已准备好 `tap0`，并且宿主机可通过该网卡向虚机提供测试规则文件。
  推荐宿主机地址：`192.168.9.2/24`，爱快管理地址：`http://192.168.9.1`。

## 运行方式

```bash
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

- `IKB_TEST_IKUAI_IMAGE`：爱快 qcow2 基础镜像路径。
- `IKB_TEST_IKUAI_URL`：爱快管理地址，默认 `http://192.168.9.1`。
- `IKB_TEST_IKUAI_USERNAME`：登录用户名，默认 `admin`。
- `IKB_TEST_IKUAI_PASSWORD`：登录密码，默认 `admin888`。
- `IKB_TEST_TAP_IF`：QEMU 第一张网卡使用的 tap 设备，默认 `tap0`。
- `IKB_TEST_FIXTURE_BIND_HOST`：宿主机测试 HTTP 服务器监听地址，默认 `0.0.0.0`。
- `IKB_TEST_FIXTURE_GUEST_HOST`：爱快虚机访问宿主机测试 HTTP 服务器时使用的地址，默认 `192.168.9.2`。
- `IKB_TEST_DEV_SCRIPT`：CLI 开发入口脚本路径，默认 `script/dev.sh`。
- `IKB_TEST_QEMU_BIN` / `IKB_TEST_QEMU_IMG_BIN`：QEMU 可执行文件路径。
- `IKB_TEST_QEMU_ACCEL`：QEMU 加速器，默认 `kvm`。
- `IKB_TEST_QEMU_MEMORY`：QEMU 内存大小，默认 `4G`。
- `IKB_TEST_QEMU_SMP`：QEMU vCPU 参数，默认 `cores=4`。
- `IKB_TEST_ARTIFACT_ROOT`：测试产物目录，默认 `apps/integration-tests/.artifacts/`。

## CI 说明

`.github/workflows/integration.yml` 会在 `pull_request` 与非 tag `push` 时运行该目录下的 smoke 测试。
该工作流现在会在 GitHub Hosted `ubuntu-latest` runner 上：

- 解压仓库内的 `.github/ikuai.qcow2.7z`
- 创建 `tap0` 并把宿主机地址设为 `192.168.9.2/24`
- 用 `1G` 内存和 `kvm` 加速启动爱快虚机
- 运行 `bash apps/integration-tests/run-smoke-test.sh`

注意：GitHub 官方文档目前明确说明，Hosted Runner 上的 nested virtualization 虽然“technically possible”，但**不属于 officially supported 能力**，属于 experimental，用于 KVM 时不保证稳定性、性能或兼容性。如果某次运行中 `/dev/kvm` 不可用，workflow 会直接失败并给出明确错误。
