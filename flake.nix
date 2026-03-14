{
  description = "iKuai Bypass development shell (Rust 主线版本)";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        bootstrapReleaseTools = pkgs.writeShellScriptBin "ikb-bootstrap-release-tools" ''
          set -euo pipefail
          cargo install cargo-binstall || true
          cargo binstall -y tauri-cli cross cargo-release
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # 基础工具
            git
            curl
            wget
            jq
            tree
            unzip
            zip
            xz

            # 构建工具
            pkg-config
            clang
            gcc
            cmake
            ninja
            perl
            gnumake

            # 系统库 (Rust/Tauri 依赖)
            openssl
            sqlite

            # GTK/WebKit (Tauri Linux GUI)
            gtk3
            glib
            cairo
            pango
            gdk-pixbuf
            atk
            libsoup_3
            webkitgtk_4_1
            libayatana-appindicator
            librsvg
            dbus
            xdg-utils
            patchelf

            # Rust 工具链
            rustup
            rustc
            cargo
            rust-analyzer
            cargo-nextest
            cargo-edit
            cargo-zigbuild
            bootstrapReleaseTools
            sccache
            mold

            # Zig (cross compilation)
            zig
            zls

            # 前端工具链 (Bun + Astro)
            nodejs_22
            bun
            typescript
            typescript-language-server
          ];

          env = {
            # Rust 编译优化
            RUSTC_WRAPPER = "sccache";
            SCCACHE_CACHE_SIZE = "10G";
            SCCACHE_DIR = "$HOME/.cache/sccache";
            CARGO_BUILD_JOBS = "16";
            RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
            RUSTUP_HOME = "$HOME/.rustup";
            CARGO_HOME = "$HOME/.cargo";
            RUSTUP_DIST_SERVER = "https://rsproxy.cn";
            RUSTUP_UPDATE_ROOT = "https://rsproxy.cn/rustup";

            # Clang
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          };

          shellHook = ''
            export PATH="$HOME/.bun/bin:$CARGO_HOME/bin:$PATH"
            export XDG_DATA_DIRS="${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:$XDG_DATA_DIRS"

            mkdir -p \
              "$HOME/.cache/sccache" \
              "$HOME/.cargo/bin"

            cat <<'EOF'
            iKuai Bypass dev shell ready (Rust 主线版本)

            项目结构:
              crates/core/     - 核心业务库
              apps/cli/        - CLI + Web 模式
              frontends/app/   - Bun + Astro 前端
              apps/gui/        - Tauri v2 GUI

            首次使用:
              rustup default stable
              rustup component add rustfmt clippy
              ikb-bootstrap-release-tools  # 安装 tauri-cli, cross 等

            常用命令:
              bash script/dev.sh cli:dev              # 运行 CLI（本体，完整功能）
              bash script/dev.sh gui:dev              # 运行 GUI (Tauri)
              bash script/dev.sh webui:dev            # 启动 Astro dev server
              bash script/dev.sh webui:build          # 构建前端 dist
            EOF
          '';
        };
      }
    );
}
