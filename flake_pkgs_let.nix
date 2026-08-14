{ nixpkgs }:
let
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
    libs = pkgList "openssl|openssl.dev|sqlite|zlib|zlib.dev|glibc.static"; # 编译链接库
    all = utils ++ net ++ archive ++ dev ++ build ++ libs;
  };

  # 前端
  jsPackages = pkgList "bun|typescript|typescript-language-server|prettierd";

  # Cloudflare Workers
  cloudflarePackages = pkgList "wrangler";

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
