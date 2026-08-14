{ nixpkgs }:
let
  readme = ''
    # flake_pkgs_let.nix —— 统一开发环境配置全集

    ## 作用
    - 所有项目共享的开发环境包配置全集，项目 flake.nix 按需引入其中的组。
    - 本文件所有副本必须保持完全一致；修改后需同步到：
      - os-config/dev-env/flake_pkgs_let.nix（模板与系统模块的数据源）
      - 各项目根目录的 flake_pkgs_let.nix

    ## 使用方式（项目 flake.nix）
    - import ./flake_pkgs_let.nix 取得 env，按需 inherit 需要的组：
        env = import ./flake_pkgs_let.nix { inherit nixpkgs; };
        inherit (env) system pkgs rustPackages desktopPackages android;
    - devShell 中 packages 组合各组的 .packages，env 组合各组的 .env。

    ## 配置组说明
    - basePackages：基础工具全集 all = utils + net + archive + dev + build + libs
      - utils 通用命令 / net 网络下载 / archive 压缩打包 / dev 开发辅助 / build 编译工具链 / libs 编译链接库
    - jsPackages：bun + typescript + tsserver + prettierd（前端）
    - cloudflarePackages：wrangler（Cloudflare Workers）
    - playwrightPackages / playwrightLibPath：chromium、google-chrome 及运行库路径
    - rustPackages：rustup + cargo 工具链 + LLVM；env 含 RUSTFLAGS / sccache 等
    - desktopPackages：Linux 桌面 GUI 依赖（GTK / WebKitGTK / 图形栈），Tauri / Flutter / Electron 通用
    - espPackages：ESP32 工具（espflash / esptool 等）+ python 串口依赖
    - docsPackages：文档工具 all = hugo + jekyll + mdbook
    - golangPackages：go + gopls + delve；env 含 GOPROXY 国内镜像
    - zigPackages：zig + zls
    - pythonPackages：python313 + uv / ruff / pyright 等
    - flutterPackages：flutter（Android 构建依赖 android 组）
    - android：Android SDK / NDK + jdk17；env 含 ANDROID_HOME / JAVA_HOME 等

    ## 快速上手
    - 复制 os-config/dev-env/flake_tpl_*.nix（按语言组合命名）到项目改名 flake.nix 即可使用。
    - 组合模板：flake_tpl_rust_tauri（Rust + Tauri + Android + Bun）、flake_tpl_all（全量）。

    ## 系统级引入（可选）
    - os-config modules/dev/dev_ext_import_all.nix 将全部包挂入系统 systemPackages：
      - 防止 nix-collect-garbage 清理开发环境包（devShell 包不在系统闭包内）
      - 仅引入 packages，不注入 env，避免污染日用环境
  '';
  system = "x86_64-linux";
  pkgs = import nixpkgs {
    inherit system;
    config = {
      allowUnfree = true;
      android_sdk.accept_license = true;
    };
  };

  pick =
    set: names:
    map (n: pkgs.lib.getAttrFromPath (pkgs.lib.splitString "." n) set) (pkgs.lib.splitString "|" names);
  pkgList = pick pkgs;

  # 基础工具：按用途细分（按需用 all 或细分字段）
  basePackages = rec {
    utils = pkgList "git|fish|hivemind|tree|ripgrep|jq|file|bc|lsof|dconf|coreutils-full"; # 通用命令行工具
    net = pkgList "curl|wget"; # 网络下载
    archive = pkgList "unzip|zip|xz|zstd|p7zip|upx"; # 压缩打包
    dev = pkgList "shellcheck|shfmt|taplo|pkg-config|perl|dpkg|protobuf"; # 开发辅助
    build = pkgList "gcc|gnumake|cmake|ninja|binutils|patchelf"; # 编译工具链
    libs = pkgList "openssl|openssl.dev|sqlite|zlib|zlib.dev"; # 编译链接库（不放 glibc.static：其 lib 会进 NIX_LDFLAGS，链接器误用 libc.a 导致 Rust 程序崩溃）
    all = utils ++ net ++ archive ++ dev ++ build ++ libs;
  };

  # 前端
  jsPackages = pkgList "bun|typescript|typescript-language-server|prettierd";

  # Cloudflare Workers（wrangler 已从 nix 移除，改用 bunx/npx wrangler 按需调用）
  cloudflarePackages = [ ];

  # Playwright
  playwrightLibPath = pkgs.lib.makeLibraryPath (
    pick pkgs "nspr|nss|cups|expat|libxcb|libXcomposite|libXdamage|libgbm|systemd|alsa-lib"
  );
  playwrightPackages = pkgList "chromium|google-chrome";
  # RustBase 使用rustup 虽然需要手动安装，但是nix表达更简单
  rustPackages = rec {
    libPath = pkgs.lib.makeLibraryPath (pkgList "openssl|curl|zlib|stdenv.cc.cc.lib");
    serverLibPath = pkgs.lib.makeLibraryPath (pkgList "openssl|sqlite|zlib|libffi");
    rustFlags = "-Cdebuginfo=1 -Ccodegen-units=1 -Clink-arg=-fuse-ld=lld -Csplit-debuginfo=packed -Clink-arg=-Wl,-rpath,${serverLibPath} -Clink-arg=-Wl,-rpath,${desktopPackages.libPath}";
    packages = pkgList "rustup|cargo-tauri|sccache|mold|cargo-edit|cargo-nextest|cargo-binstall|cargo-release|lldb|llvmPackages_19.clang|llvmPackages_19.libclang|llvmPackages_19.libllvm|llvmPackages_19.lld|gcc|gnumake|cmake|ninja|binutils|patchelf";
    env = {
      RUSTFLAGS = rustFlags;
      RUSTC_WRAPPER = "sccache";
      SCCACHE_CACHE_SIZE = "30G";
      CARGO_BUILD_JOBS = "12";
      CARGO_INCREMENTAL = "0";
      CARGO_PROFILE_DEV_INCREMENTAL = "false";
      CARGO_PROFILE_DEV_DEBUG = "0";
      CARGO_CACHE_RUSTC_INFO = "0";
      RUSTC_CODEGEN_UNITS = "1";
      LIBCLANG_PATH = "${pkgs.llvmPackages_19.libclang.lib}/lib";
      LIBRARY_PATH = "${serverLibPath}:${desktopPackages.libPath}";
      LD_LIBRARY_PATH = libPath;
    };
  };
  # Linux 桌面 GUI 应用依赖（Tauri/Flutter/Electron 通用）
  desktopPackages = {
    libPath = pkgs.lib.makeLibraryPath (
      pkgList "libglvnd|mesa|libgbm|wayland|libx11|libxrandr|libxrender|libxcursor|libxinerama|libxi|libxext|libxfixes|libxcb|libXcomposite|libXdamage|glib|gtk3|cairo|pango|gdk-pixbuf|atk|harfbuzz|pcre|gst_all_1.gstreamer|gst_all_1.gst-plugins-base|gst_all_1.gst-plugins-good|gst_all_1.gst-plugins-bad|gst_all_1.gst-plugins-ugly|gst_all_1.gst-libav|libsoup_3|webkitgtk_4_1|dbus|at-spi2-core|libxkbcommon|libayatana-appindicator|librsvg|nss|nspr|cups|expat|systemd|udev|alsa-lib"
    );
    packages = pkgList "webkitgtk_4_1|webkitgtk_4_1.dev|libsoup_3|libsoup_3.dev|gtk3|gtk3.dev|glib|glib.dev|cairo.dev|pango.dev|gdk-pixbuf.dev|atk.dev|harfbuzz.dev|libepoxy.dev|librsvg|librsvg.dev|at-spi2-core.dev|dbus.dev|libayatana-appindicator|glib-networking|gsettings-desktop-schemas|libglvnd|libglvnd.dev|mesa|libxkbcommon.dev|wayland.dev|libx11.dev|libxext.dev|libxi.dev|libxrandr.dev|libxrender.dev|libxcursor.dev|libxinerama.dev|libxfixes.dev|libxcb|libXcomposite|libXdamage|libxtst|libxxf86vm|libdrm|libffi.dev|pcre.dev|openssl|openssl.dev|sqlite.dev|zlib.dev|expat|nss|nspr|cups|udev|alsa-lib|gst_all_1.gstreamer|gst_all_1.gst-plugins-base|gst_all_1.gst-plugins-good|gst_all_1.gst-plugins-bad|gst_all_1.gst-plugins-ugly|gst_all_1.gst-libav|dconf";
    env = {
      GIO_EXTRA_MODULES = "${pkgs.glib-networking}/lib/gio/modules:${pkgs.dconf.lib}/lib/gio/modules";
      GST_PLUGIN_SYSTEM_PATH_1_0 = (
        pkgs.lib.concatStringsSep ":" (
          map (pkg: "${pkgs.lib.getOutput "out" pkg}/lib/gstreamer-1.0") (
            pick pkgs.gst_all_1 "gstreamer|gst-plugins-base|gst-plugins-good|gst-plugins-bad|gst-plugins-ugly|gst-libav"
          )
        )
      );
      GST_PLUGIN_SCANNER_1_0 = "${pkgs.lib.getOutput "out" pkgs.gst_all_1.gstreamer}/libexec/gstreamer-1.0/gst-plugin-scanner";
      WEBKIT_DISABLE_COMPOSITING_MODE = "1";
    };
  };
  # exp32工具
  espPackages = {
    tools = pkgList "espflash|esptool|ldproxy|mpremote|minicom|picocom|libxml2";
    pythonDeps =
      ps: with ps; [
        pyserial
      ];
  };

  #文档工具
  # 文档工具（hugo / jekyll / mdbook，按需用 all 或细分字段）
  docsPackages = rec {
    hugo = pkgList "hugo|go";
    jekyll = [ (pkgs.ruby.withPackages (_: [ pkgs.bundler ])) ];
    mdbook = pkgList "mdbook";
    all = hugo ++ jekyll ++ mdbook;
  };

  # Go（含国内镜像配置）
  golangPackages = {
    packages = pkgList "go|gopls|delve";
    env = {
      GOPROXY = "https://goproxy.cn,direct";
      GOSUMDB = "sum.golang.org";
    };
  };

  # Zig
  zigPackages = pkgList "zig|zls";

  pythonPackages = {
    packages = [
      (pkgs.python313.withPackages (
        ps: (espPackages.pythonDeps ps) ++ (pick ps "pip|tkinter|json5|protobuf|click|pyyaml")
      ))
    ]
    ++ pkgList "uv|ruff|python3Packages.debugpy|pyright";
  };
  # Flutter（Android 构建依赖 android 组）
  flutterPackages = pkgList "flutter";

  # 安卓编译 tauri 和 flutter必须
  android = rec {
    sdk =
      (pkgs.androidenv.composeAndroidPackages {
        cmdLineToolsVersion = "latest";
        platformToolsVersion = "latest";
        buildToolsVersions = pkgs.lib.splitString "|" "35.0.0|36.1.0";
        platformVersions = pkgs.lib.splitString "|" "34|35|36";
        includeNDK = true;
        ndkVersion = "28.2.13676358";
        includeCmake = true;
        includeEmulator = false;
        includeSources = false;
      }).androidsdk;
    home = "${sdk}/libexec/android-sdk";
    ndk = "${home}/ndk/28.2.13676358";
    packages = [ sdk ] ++ pkgList "jdk17|cargo-ndk|usbutils|scrcpy";
    env = {
      ANDROID_HOME = home;
      ANDROID_SDK_ROOT = home;
      ANDROID_NDK = ndk;
      ANDROID_NDK_HOME = ndk;
      JAVA_HOME = "${pkgs.jdk17}";
      GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${home}/build-tools/35.0.0/aapt2";
    };
  };
in
{
  inherit
    readme
    system
    pkgs
    basePackages
    jsPackages
    cloudflarePackages
    playwrightLibPath
    playwrightPackages
    rustPackages
    desktopPackages
    espPackages
    docsPackages
    golangPackages
    zigPackages
    pythonPackages
    flutterPackages
    android
    ;
}
