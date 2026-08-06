{
  # devShell 的 WebKitGTK TLS 依赖系统服务 services.gnome.glib-networking（os-config dev_webkitgtk.nix）
  description = "Rust CLI / Tauri Linux x86 GUI / Bun Astro 前端 / Jekyll 文档 开发编译环境";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config = {
          allowUnfree = true;
          android_sdk.accept_license = true;
        };
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

      androidSdk = (pkgs.androidenv.composeAndroidPackages {
        cmdLineToolsVersion = "latest";
        platformToolsVersion = "latest";
        buildToolsVersions = [ "35.0.0" "36.1.0" ];
        platformVersions = [ "34" "35" "36" ];
        includeNDK = true;
        ndkVersion = "28.2.13676358";
        includeCmake = true;
        includeEmulator = false;
        includeSources = false;
      }).androidsdk;
      androidHome = "${androidSdk}/libexec/android-sdk";
      androidNdk = "${androidHome}/ndk/28.2.13676358";

      libraryPath = pkgs.lib.makeLibraryPath (with pkgs; [
        openssl
        sqlite
        zlib
        libglvnd
        mesa
        libgbm
        wayland
        libx11
        libxrandr
        libxrender
        libxcursor
        libxinerama
        libxi
        libxext
        libxfixes
        libxcb
        libXcomposite
        libXdamage
        glib
        gtk3
        cairo
        pango
        gdk-pixbuf
        atk
        harfbuzz
        pcre
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-good
        gst_all_1.gst-plugins-bad
        gst_all_1.gst-plugins-ugly
        gst_all_1.gst-libav
        libsoup_3
        webkitgtk_4_1
        dbus
        at-spi2-core
        libxkbcommon
        libayatana-appindicator
        librsvg
        libffi
        nss
        nspr
        cups
        expat
        systemd
        udev
        alsa-lib
      ]);

      rustFlags = "-Cdebuginfo=1 -Ccodegen-units=1 -Clink-arg=-fuse-ld=lld -Csplit-debuginfo=packed -Clink-arg=-Wl,-rpath,${libraryPath}";
      crossRustFlags = "-Cdebuginfo=1 -Ccodegen-units=1 -Csplit-debuginfo=packed";

      bootstrapReleaseTools = pkgs.writeShellScriptBin "ikb-bootstrap-release-tools" ''
        set -euo pipefail
        cargo binstall -y tauri-cli cross cargo-dist
      '';

      basePackages = with pkgs; [
        git
        curl
        wget
        jq
        tree
        unzip
        zip
        xz
        ripgrep
        lsof
        bc
        hivemind
        taplo
      ];

      serverLibPackages = with pkgs; [
        gcc
        gnumake
        cmake
        ninja
        binutils
        pkg-config
        perl
        dpkg
        zstd
        file
        patchelf
      ];

      jsPackages = with pkgs; [
        bun
        nodejs_26
        pnpm
        typescript
        typescript-language-server
        prettier
      ];

      rustPackages = with pkgs; [
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
        zig
        llvmPackages_19.clang
        llvmPackages_19.libclang
        llvmPackages_19.libllvm
        llvmPackages_19.lld
      ];

      androidPackages = with pkgs; [
        jdk17
        androidSdk
      ];

      jekyllPackages = with pkgs; [
        (ruby.withPackages (ps: with ps; [ bundler ]))
      ];

      playwrightPackages = with pkgs; [
        chromium
      ];

      tauriNative = with pkgs; [
        webkitgtk_4_1
        webkitgtk_4_1.dev
        libsoup_3
        libsoup_3.dev
        gtk3
        gtk3.dev
        glib
        glib.dev
        cairo.dev
        pango.dev
        gdk-pixbuf.dev
        atk.dev
        harfbuzz.dev
        libepoxy.dev
        librsvg
        librsvg.dev
        at-spi2-core.dev
        dbus.dev
        libayatana-appindicator
        glib-networking
        gsettings-desktop-schemas
      ];

      guiNative = with pkgs; [
        libglvnd
        libglvnd.dev
        mesa
        libxkbcommon.dev
        wayland.dev
        libx11.dev
        libxext.dev
        libxi.dev
        libxrandr.dev
        libxrender.dev
        libxcursor.dev
        libxinerama.dev
        libxfixes.dev
        libxcb
        libXcomposite
        libXdamage
        libxtst
        libxxf86vm
        libdrm
        libffi.dev
        pcre.dev
        openssl
        openssl.dev
        sqlite.dev
        zlib.dev
        expat
        nss
        nspr
        cups
        udev
        alsa-lib
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-good
        gst_all_1.gst-plugins-bad
        gst_all_1.gst-plugins-ugly
        gst_all_1.gst-libav
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages =
          basePackages
          ++ serverLibPackages
          ++ jsPackages
          ++ rustPackages
          ++ androidPackages
          ++ jekyllPackages
          ++ playwrightPackages
          ++ tauriNative
          ++ guiNative
          ++ [ pkgs.fish ];

        env = {
          ANDROID_HOME = androidHome;
          ANDROID_SDK_ROOT = androidHome;
          ANDROID_NDK = androidNdk;
          ANDROID_NDK_HOME = androidNdk;
          JAVA_HOME = "${pkgs.jdk17}";
          GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidHome}/build-tools/35.0.0/aapt2";

          RUSTFLAGS = rustFlags;
          RUSTC_WRAPPER = "sccache";
          SCCACHE_CACHE_SIZE = "30G";
          CARGO_BUILD_JOBS = "16";
          CARGO_INCREMENTAL = "0";
          CARGO_PROFILE_DEV_INCREMENTAL = "false";
          CARGO_PROFILE_DEV_DEBUG = "0";
          CARGO_CACHE_RUSTC_INFO = "0";
          RUSTC_CODEGEN_UNITS = "1";
          LIBCLANG_PATH = "${pkgs.llvmPackages_19.libclang.lib}/lib";
          LIBRARY_PATH = libraryPath;
          CROSS_RUSTFLAGS = crossRustFlags;
          RUSTUP_HOME = "/home/y/.rustup";
          CARGO_HOME = "/home/y/.cargo";
          RUSTUP_DIST_SERVER = "https://rsproxy.cn";
          RUSTUP_UPDATE_ROOT = "https://rsproxy.cn/rustup";
          PLAYWRIGHT_LD_LIBRARY_PATH = playwrightLibraryPath;

          GIO_EXTRA_MODULES = "${pkgs.glib-networking}/lib/gio/modules:${pkgs.dconf.lib}/lib/gio/modules";

          GST_PLUGIN_SYSTEM_PATH_1_0 = (pkgs.lib.concatStringsSep ":" (map (pkg: "${pkgs.lib.getOutput "out" pkg}/lib/gstreamer-1.0") (with pkgs.gst_all_1; [ gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav ])));
          GST_PLUGIN_SCANNER_1_0 = "${pkgs.lib.getOutput "out" pkgs.gst_all_1.gstreamer}/libexec/gstreamer-1.0/gst-plugin-scanner";

          WEBKIT_DISABLE_COMPOSITING_MODE = "1";
        };

        shellHook = ''
          export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/build-tools/35.0.0:$PATH"
          export PATH="$CARGO_HOME/bin:$PATH"
          export SCCACHE_DIR="$HOME/.cache/sccache"

          if [ -n "$LD_LIBRARY_PATH" ]; then
            export LD_LIBRARY_PATH="$PLAYWRIGHT_LD_LIBRARY_PATH:$LD_LIBRARY_PATH"
          else
            export LD_LIBRARY_PATH="$PLAYWRIGHT_LD_LIBRARY_PATH"
          fi

          __CROSS_TARGETS=(aarch64-unknown-linux-gnu x86_64-unknown-linux-musl wasm32-unknown-unknown)
          __MISSING=()
          for t in "''${__CROSS_TARGETS[@]}"; do
            rustup target list --installed 2>/dev/null | grep -qxF "$t" || __MISSING+=("$t")
          done
          if [ "''${#__MISSING[@]}" -gt 0 ]; then
            echo "  [rustup] 安装缺失 targets: ''${__MISSING[*]}（首次运行需要网络下载）"
            rustup target add "''${__MISSING[@]}" 2>/dev/null || echo "  [warn] target 安装失败，请手动执行: rustup target add ''${__MISSING[*]}"
          fi
          unset __CROSS_TARGETS __MISSING

          if command -v cargo-tauri >/dev/null 2>&1; then
            CARGO_TAURI_BIN="$(command -v cargo-tauri)"
            CARGO_TAURI_VER="$(cargo-tauri --version 2>/dev/null || echo n/a)"
          else
            CARGO_TAURI_BIN="(未找到，请先 cargo install tauri-cli --locked --version 2.11.4)"
            CARGO_TAURI_VER="n/a"
          fi

          if ! rustup show active-toolchain >/dev/null 2>&1; then
            echo "  [提示] 未设置默认 Rust 工具链，请执行: rustup default stable"
          fi

          mkdir -p "$HOME/.cache/sccache" "$CARGO_HOME/bin"

          echo ""
          echo "== iKuai Bypass devShell =="
          echo "  rustc         = $(rustc --version 2>/dev/null || echo '未安装（rustup default stable）')"
          echo "  bun           = $(bun --version 2>/dev/null)"
          echo "  ruby/bundle   = $(ruby --version 2>/dev/null) / $(bundle --version 2>/dev/null || echo '?')"
          echo "  cargo-tauri   = $CARGO_TAURI_BIN ($CARGO_TAURI_VER)"
          echo "  zig / cargo-zigbuild = $(zig version 2>/dev/null) / $(cargo-zigbuild --version 2>/dev/null)"
          echo "  ANDROID_HOME  = $ANDROID_HOME"
          echo "  ANDROID_NDK   = $ANDROID_NDK"
          echo "  JAVA_HOME     = $JAVA_HOME"
          echo ""
          echo "项目结构:"
          echo "  crates/core/     - 核心业务库"
          echo "  apps/cli/        - CLI + Web 模式"
          echo "  frontends/app/   - Bun + Astro 前端"
          echo "  apps/gui/        - Tauri v2 GUI"
          echo ""
          echo "常用命令:"
          echo "  bash script/dev.sh cli:dev              # 运行 CLI（本体，完整功能）"
          echo "  bash script/dev.sh gui:dev              # 运行 GUI (Tauri)"
          echo "  bash script/dev.sh webui:dev            # 启动 Astro dev server"
          echo "  bash script/dev.sh webui:build          # 构建前端 dist"
          echo ""
          echo "Jekyll 站点（docs/）:"
          echo "  bundle install && bundle exec jekyll serve   # 本地预览（localhost:4000）"
          # 默认落进 fish（带专门 dev 主题，与系统 bash/fish 明确区分）
          # 仅在交互式 TTY 下 exec，命令行模式（nix develop -c）保留原 shell
          if [ -t 0 ] && command -v fish >/dev/null 2>&1; then
            export __FISH_DEVSHELL=1
            exec fish
          fi
        '';
      };
    };
}
