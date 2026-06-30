{
  description = "iKuai Bypass development shell (Rust 主线版本)";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-26.05";
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

        playwrightRuntimeLibs = [
          pkgs.nspr
          pkgs.nss
          pkgs.cups
          pkgs.expat
          pkgs.libxcb
          pkgs.libXcomposite
          pkgs.libXdamage
          pkgs.libgbm
          pkgs.systemd
          pkgs.alsa-lib
        ];

        playwrightLibraryPath = pkgs.lib.makeLibraryPath playwrightRuntimeLibs;

        bootstrapReleaseTools = pkgs.writeShellScriptBin "ikb-bootstrap-release-tools" ''
          set -euo pipefail
          cargo binstall -y tauri-cli cross cargo-dist
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
            libclang
            cmake
            ninja
            perl
            gnumake

            # 系统库 (Rust/Tauri 依赖)
            openssl
            openssl.dev
            sqlite
            zlib
            zlib.dev
            glibc.static

            # GTK/WebKit (Tauri Linux GUI)
            gtk3
            gtk3.dev
            glib
            glib.dev
            cairo
            cairo.dev
            pango
            pango.dev
            gdk-pixbuf
            gdk-pixbuf.dev
            atk
            atk.dev
            gsettings-desktop-schemas
            libsoup_3
            libsoup_3.dev
            webkitgtk_4_1
            webkitgtk_4_1.dev
            libayatana-appindicator
            librsvg
            librsvg.dev
            dbus
            dbus.dev
            libepoxy
            libepoxy.dev
            at-spi2-core
            at-spi2-core.dev
            harfbuzz
            harfbuzz.dev
            xdg-utils
            patchelf
            chromium

            nspr
            nss
            cups
            expat
            libxcb
            libXcomposite
            libXdamage
            libgbm
            alsa-lib

            # Rust 工具链 (rustup 管理 rustc/cargo 版本，避免与 nixpkgs 版本冲突)
            rustup
            cargo-binstall
            cargo-edit
            cargo-release
            cargo-nextest
            cargo-zigbuild
            rust-analyzer
            bootstrapReleaseTools
            sccache
            mold


            # 前端工具链 (Bun + Astro)
            nodejs_26
            bun
            pnpm
            typescript
            typescript-language-server
            prettier

            # 通用开发辅助工具
            ripgrep
            taplo
            file
            lsof
            bc
            hivemind
          ];

          env = {
            # Rust 编译缓存
            RUSTC_WRAPPER = "sccache";
            SCCACHE_CACHE_SIZE = "10G";
            CARGO_BUILD_JOBS = "16";
            # 清空宿主机继承和 nix 包装器注入的 RUSTFLAGS
            RUSTFLAGS = "";

            # Clang
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            PLAYWRIGHT_LD_LIBRARY_PATH = playwrightLibraryPath;
          };

          shellHook = ''
            export CARGO_HOME="$HOME/.cargo"
            export SCCACHE_DIR="$HOME/.cache/sccache"
            export PATH="$HOME/.bun/bin:$PATH:$CARGO_HOME/bin"

            # Nix gcc/clang 包装器注入的环境变量会导致二进制链接到有问题的
            # libgcc_s.so.1（gcc-15.2.0 IFUNC bug），清除它们让宿主机 gcc 直接工作
            unset NIX_CC NIX_CC_WRAPPER_TARGET_HOST_x86_64_unknown_linux_gnu
            unset NIX_LDFLAGS NIX_CFLAGS_COMPILE NIX_CFLAGS_LINK

            if [ -n "$LD_LIBRARY_PATH" ]; then
              export LD_LIBRARY_PATH="$PLAYWRIGHT_LD_LIBRARY_PATH:$LD_LIBRARY_PATH"
            else
              export LD_LIBRARY_PATH="$PLAYWRIGHT_LD_LIBRARY_PATH"
            fi
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
