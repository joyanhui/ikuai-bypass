{
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

      pick =
        set: names:
        map (n: pkgs.lib.getAttrFromPath (pkgs.lib.splitString "." n) set) (pkgs.lib.splitString "|" names);
      pkgList = pick pkgs;

      basePackages = pkgList "git|curl|wget|jq|tree|unzip|zip|xz|ripgrep|lsof|bc|hivemind|taplo|fish";

      bootstrapReleaseTools = pkgs.writeShellScriptBin "ikb-bootstrap-release-tools" ''
        set -euo pipefail
        cargo binstall -y tauri-cli cross cargo-dist
      '';

      playwrightLibPath = pkgs.lib.makeLibraryPath (
        pick pkgs "nspr|nss|cups|expat|libxcb|libXcomposite|libXdamage|libgbm|systemd|alsa-lib"
      );

      rustPackages = rec {
        libPath = pkgs.lib.makeLibraryPath (pkgList "openssl|curl|zlib|stdenv.cc.cc.lib");
        serverLibPath = pkgs.lib.makeLibraryPath (pkgList "openssl|sqlite|zlib|libffi");
        rustFlags = "-Cdebuginfo=1 -Ccodegen-units=1 -Clink-arg=-fuse-ld=lld -Csplit-debuginfo=packed -Clink-arg=-Wl,-rpath,${serverLibPath} -Clink-arg=-Wl,-rpath,${tauriPackages.libPath}";
        packages = pkgList "rustup|sccache|mold|cargo-edit|cargo-nextest|cargo-binstall|cargo-release|lldb|llvmPackages_19.clang|llvmPackages_19.libclang|llvmPackages_19.libllvm|llvmPackages_19.lld|gcc|gnumake|cmake|ninja|binutils|pkg-config|perl|dpkg|zstd|file|patchelf";
        env = {
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
          LIBRARY_PATH = "${serverLibPath}:${tauriPackages.libPath}";
          LD_LIBRARY_PATH = "${playwrightLibPath}:${libPath}";
        };
      };

      tauriPackages = {
        libPath = pkgs.lib.makeLibraryPath (
          pkgList "libglvnd|mesa|libgbm|wayland|libx11|libxrandr|libxrender|libxcursor|libxinerama|libxi|libxext|libxfixes|libxcb|libXcomposite|libXdamage|glib|gtk3|cairo|pango|gdk-pixbuf|atk|harfbuzz|pcre|gst_all_1.gstreamer|gst_all_1.gst-plugins-base|gst_all_1.gst-plugins-good|gst_all_1.gst-plugins-bad|gst_all_1.gst-plugins-ugly|gst_all_1.gst-libav|libsoup_3|webkitgtk_4_1|dbus|at-spi2-core|libxkbcommon|libayatana-appindicator|librsvg|nss|nspr|cups|expat|systemd|udev|alsa-lib"
        );
        packages = pkgList "cargo-tauri|webkitgtk_4_1|webkitgtk_4_1.dev|libsoup_3|libsoup_3.dev|gtk3|gtk3.dev|glib|glib.dev|cairo.dev|pango.dev|gdk-pixbuf.dev|atk.dev|harfbuzz.dev|libepoxy.dev|librsvg|librsvg.dev|at-spi2-core.dev|dbus.dev|libayatana-appindicator|glib-networking|gsettings-desktop-schemas|libglvnd|libglvnd.dev|mesa|libxkbcommon.dev|wayland.dev|libx11.dev|libxext.dev|libxi.dev|libxrandr.dev|libxrender.dev|libxcursor.dev|libxinerama.dev|libxfixes.dev|libxcb|libXcomposite|libXdamage|libxtst|libxxf86vm|libdrm|libffi.dev|pcre.dev|openssl|openssl.dev|sqlite.dev|zlib.dev|expat|nss|nspr|cups|udev|alsa-lib|gst_all_1.gstreamer|gst_all_1.gst-plugins-base|gst_all_1.gst-plugins-good|gst_all_1.gst-plugins-bad|gst_all_1.gst-plugins-ugly|gst_all_1.gst-libav";
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

      jsPackages = pkgList "bun|typescript|typescript-language-server|prettier";

      jekyllPackages = [ (pkgs.ruby.withPackages (_: [ pkgs.bundler ])) ];

      playwrightPackages = pkgList "chromium";

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
        packages = [ sdk ] ++ pkgList "jdk17|usbutils|scrcpy";
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
      devShells.${system}.default = pkgs.mkShell {
        packages =
          rustPackages.packages
          ++ tauriPackages.packages
          ++ android.packages
          ++ jsPackages
          ++ jekyllPackages
          ++ playwrightPackages
          ++ basePackages
          ++ [ bootstrapReleaseTools ];

        env = rustPackages.env // tauriPackages.env // android.env;

        shellHook = ''
          export PATH="$HOME/.cargo/bin:$PATH"
          export SCCACHE_DIR="$HOME/.cache/sccache"

          echo "==========================================================="
          echo "== iKuai Bypass devShell =="
          echo "  Rust 工具链由 rustup 管理，首次进入请执行："
          echo "    rustup default stable"
          echo "  bun 依赖："
          echo "    bun install"
          echo "  Jekyll 文档本地预览："
          echo "    bundle install && bundle exec jekyll serve"
          echo "==========================================================="
          if [ -t 0 ] && command -v fish >/dev/null 2>&1; then
            export __FISH_DEVSHELL=1
            exec fish
          fi
        '';
      };
    };
}
