#!/bin/bash
# iKuai Bypass ipkg 打包脚本 / Build script for iKuai v4 ipkg package
# 用法 / Usage: cd packaging/ikuai-ipkg && bash build-ipkg.sh
#
# 复用根目录 Dockerfile，本地构建时先用临时容器编译产物，再组装镜像。
# 在 CI 中可直接使用已有 build-cli / build-frontend 产物，避免二次编译。
# Reuses root Dockerfile. For local builds, compiles artifacts in temp
# containers first, then assembles the final image. In CI, pre-built
# artifacts from build-cli / build-frontend can be used directly.
#
# 版本号自动从 apps/cli/Cargo.toml 读取（唯一来源），并仅在 staging 渲染 manifest。
# Version is auto-read from apps/cli/Cargo.toml (single source of truth) and
# only rendered into the staged manifest.
# 输出 / Output: ikuai-bypass-x86_64.ipkg

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
STAGE_DIR="$SCRIPT_DIR/.stage"
PACKAGE_TEMPLATE_DIR="$SCRIPT_DIR/ikuai-bypass"
PACKAGE_STAGE_DIR="$STAGE_DIR/package/ikuai-bypass"
MANIFEST_TEMPLATE="$PACKAGE_TEMPLATE_DIR/manifest.template.json"
RENDER_MANIFEST_SCRIPT="$SCRIPT_DIR/render-manifest.sh"

normalize_ipkg_version() {
  local raw="${1:-}"
  if [[ "$raw" =~ ^([0-9]+\.[0-9]+\.[0-9]+)([-+].*)?$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
    return
  fi
  printf '%s' "$raw"
}

# 从 Cargo.toml 提取版本号，去掉预发布后缀以满足 ipkg 的 X.Y.Z 格式
# Extract version from Cargo.toml, strip prerelease suffix for ipkg semver X.Y.Z
RAW_VERSION=$(grep '^version' "$PROJECT_DIR/apps/cli/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
VERSION=$(normalize_ipkg_version "$RAW_VERSION")
if [ -z "$VERSION" ]; then
  echo "ERROR: could not extract version from apps/cli/Cargo.toml"
  exit 1
fi

echo "=== 构建 ikuai-bypass v${VERSION} ipkg (原始版本: ${RAW_VERSION}) ==="

PACKAGE_NAME="ikuai-bypass-x86_64.ipkg"

# 步骤 0：同步图标 / Step 0: Sync icons from the GUI source icon
bash "$PROJECT_DIR/apps/gui/scripts/sync-icons.sh" ipkg-only

# 步骤 1：准备暂存目录并渲染 manifest.json / Step 1: Prepare staging and render manifest.json
echo "[1/6] 准备暂存目录并渲染 manifest.json..."

# Why/为什么: 最终 ipkg 需要成品 manifest.json，但源码树只保留模板，避免本地/CI 把仓库改脏。
# English: The final ipkg needs a real manifest.json, while the repo keeps a template to avoid dirtying the tree.
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/docker/bin/linux-amd64" "$STAGE_DIR/docker/frontends/app" "$PACKAGE_STAGE_DIR"
cp -R "$PACKAGE_TEMPLATE_DIR/." "$PACKAGE_STAGE_DIR/"
rm -f "$PACKAGE_STAGE_DIR/manifest.template.json" "$PACKAGE_STAGE_DIR/docker_image.tar.gz"
bash "$RENDER_MANIFEST_SCRIPT" "$MANIFEST_TEMPLATE" "$PACKAGE_STAGE_DIR/manifest.json" "$VERSION"

# 准备暂存目录（模拟根 Dockerfile 所需的目录结构）
# Prepare staging dir (mirror the layout root Dockerfile expects)

# 步骤 2：编译前端（临时容器）/ Step 2: Build frontend in temp container
echo "[2/6] 构建前端..."
docker build --target frontend -t ikb-ipkg-frontend -f - "$PROJECT_DIR" <<'DOCKERFILE'
FROM oven/bun:1-alpine AS frontend
WORKDIR /build
COPY frontends/app/package.json frontends/app/bun.lock* ./
RUN bun install --frozen-lockfile || bun install
COPY frontends/app/ ./
RUN bun run build
DOCKERFILE

# 从临时镜像中提取前端产物 / Extract frontend dist from temp image
CID=$(docker create ikb-ipkg-frontend true)
docker cp "$CID:/build/dist" "$STAGE_DIR/docker/frontends/app/dist"
docker rm "$CID" >/dev/null

# 步骤 3：编译 CLI 二进制（临时容器）/ Step 3: Build CLI binary in temp container
echo "[3/6] 编译 CLI 二进制..."
docker build --target builder -t ikb-ipkg-builder -f - "$PROJECT_DIR" <<'DOCKERFILE'
FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.lock ./
RUN printf '[workspace]\nresolver = "2"\nmembers = ["crates/core", "apps/cli"]\n' > Cargo.toml
COPY crates/ crates/
COPY apps/cli/ apps/cli/
RUN cargo build --release -p ikb-cli \
    && cp target/release/ikb-cli target/release/ikuai-bypass \
    && strip target/release/ikuai-bypass || true
DOCKERFILE

# 从临时镜像中提取二进制 / Extract binary from temp image
CID=$(docker create ikb-ipkg-builder true)
docker cp "$CID:/build/target/release/ikuai-bypass" "$STAGE_DIR/docker/bin/linux-amd64/ikuai-bypass"
docker rm "$CID" >/dev/null

# 步骤 4：用根 Dockerfile 组装最终镜像（DOCKER_BUILDKIT=0 兼容 iKuai Docker 18.09）
# Step 4: Assemble final image with root Dockerfile (DOCKER_BUILDKIT=0 for iKuai compat)
echo "[4/6] 组装 Docker 镜像..."
cp "$PROJECT_DIR/Dockerfile" "$PROJECT_DIR/packaging/docker/docker-entrypoint.sh" "$PROJECT_DIR/config.yml" "$STAGE_DIR/"
cd "$STAGE_DIR"
DOCKER_BUILDKIT=0 docker build --build-arg TARGETPLATFORM=linux/amd64 -t ikuai-bypass:ikuai .

# 步骤 5：导出镜像为离线安装包 / Step 5: Export image as offline package
echo "[5/6] 导出 Docker 镜像..."
docker save ikuai-bypass:ikuai | gzip > "$PACKAGE_STAGE_DIR/docker_image.tar.gz"
IMAGE_SIZE=$(du -h "$PACKAGE_STAGE_DIR/docker_image.tar.gz" | cut -f1)
echo "    镜像大小 / Image size: ${IMAGE_SIZE}"

# 步骤 6：打包 ipkg / Step 6: Pack ipkg
echo "[6/6] 打包 ipkg..."
tar -czf "$SCRIPT_DIR/${PACKAGE_NAME}" -C "$STAGE_DIR/package" ikuai-bypass/
IPKG_SIZE=$(du -h "$SCRIPT_DIR/${PACKAGE_NAME}" | cut -f1)

# 清理临时文件 / Clean up
rm -rf "$STAGE_DIR"

echo ""
echo "=== 完成 / Done ==="
echo "输出 / Output: packaging/ikuai-ipkg/${PACKAGE_NAME} (${IPKG_SIZE})"
echo ""
echo "安装方式 / Install on iKuai v4:"
echo "  Web: 高级应用 → 应用市场 → 本地安装"
