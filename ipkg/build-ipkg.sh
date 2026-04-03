#!/bin/bash
# iKuai Bypass ipkg 打包脚本 / Build script for iKuai v4 ipkg package
# 用法 / Usage: cd ipkg && bash build-ipkg.sh
#
# 复用根目录 Dockerfile，本地构建时先用临时容器编译产物，再组装镜像。
# 在 CI 中可直接使用已有 build-cli / build-frontend 产物，避免二次编译。
# Reuses root Dockerfile. For local builds, compiles artifacts in temp
# containers first, then assembles the final image. In CI, pre-built
# artifacts from build-cli / build-frontend can be used directly.
#
# 版本号自动从 apps/cli/Cargo.toml 读取（唯一来源）
# Version is auto-read from apps/cli/Cargo.toml (single source of truth)
# 输出 / Output: ikuai-bypass-<version>.ipkg

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
STAGE_DIR="$SCRIPT_DIR/.stage"

# 从 Cargo.toml 提取版本号，去掉预发布后缀以满足 ipkg 的 X.Y.Z 格式
# Extract version from Cargo.toml, strip prerelease suffix for ipkg semver X.Y.Z
RAW_VERSION=$(grep '^version' "$PROJECT_DIR/apps/cli/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
VERSION=$(echo "$RAW_VERSION" | sed 's/-.*//')
if [ -z "$VERSION" ]; then
  echo "ERROR: could not extract version from apps/cli/Cargo.toml"
  exit 1
fi

echo "=== 构建 ikuai-bypass v${VERSION} ipkg (原始版本: ${RAW_VERSION}) ==="

# 步骤 1：同步版本号到 manifest.json / Step 1: Sync version to manifest.json
echo "[1/6] 同步版本号到 manifest.json..."
sed -i "s/\"version\": *\"[^\"]*\"/\"version\": \"${VERSION}\"/" "$SCRIPT_DIR/ikuai-bypass/manifest.json"

# 准备暂存目录（模拟根 Dockerfile 所需的目录结构）
# Prepare staging dir (mirror the layout root Dockerfile expects)
rm -rf "$STAGE_DIR" && mkdir -p "$STAGE_DIR/docker/bin/linux-amd64" "$STAGE_DIR/docker/frontends/app"

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
cp "$PROJECT_DIR/Dockerfile" "$PROJECT_DIR/docker-entrypoint.sh" "$PROJECT_DIR/config.yml" "$STAGE_DIR/"
cd "$STAGE_DIR"
DOCKER_BUILDKIT=0 docker build --build-arg TARGETPLATFORM=linux/amd64 -t ikuai-bypass:ikuai .

# 步骤 5：导出镜像为离线安装包 / Step 5: Export image as offline package
echo "[5/6] 导出 Docker 镜像..."
docker save ikuai-bypass:ikuai | gzip > "$SCRIPT_DIR/ikuai-bypass/docker_image.tar.gz"
IMAGE_SIZE=$(du -h "$SCRIPT_DIR/ikuai-bypass/docker_image.tar.gz" | cut -f1)
echo "    镜像大小 / Image size: ${IMAGE_SIZE}"

# 步骤 6：打包 ipkg / Step 6: Pack ipkg
echo "[6/6] 打包 ipkg..."
cd "$SCRIPT_DIR"
tar -czf "ikuai-bypass-${VERSION}.ipkg" ikuai-bypass/
IPKG_SIZE=$(du -h "ikuai-bypass-${VERSION}.ipkg" | cut -f1)

# 清理临时文件 / Clean up
rm -rf "$STAGE_DIR"
rm -f "$SCRIPT_DIR/ikuai-bypass/docker_image.tar.gz"

echo ""
echo "=== 完成 / Done ==="
echo "输出 / Output: ipkg/ikuai-bypass-${VERSION}.ipkg (${IPKG_SIZE})"
echo ""
echo "安装方式 / Install on iKuai v4:"
echo "  Web: 高级应用 → 应用市场 → 本地安装"
