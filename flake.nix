{
  description = "Rust CLI / Tauri Linux x86 GUI / Bun Astro 前端 / Jekyll 文档 开发编译环境";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };

  outputs =
    { nixpkgs, ... }:
    let
      env = import ./flake_pkgs_let.nix { inherit nixpkgs; };
      inherit (env)
        system
        pkgs
        basePackages
        rustPackages
        desktopPackages
        android
        jsPackages
        docsPackages
        playwrightPackages
        playwrightLibPath
        ;

      bootstrapReleaseTools = pkgs.writeShellScriptBin "ikb-bootstrap-release-tools" ''
        set -euo pipefail
        cargo binstall -y tauri-cli cross cargo-dist
      '';
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages =
          basePackages.all
          ++ rustPackages.packages
          ++ desktopPackages.packages
          ++ android.packages
          ++ jsPackages
          ++ docsPackages.jekyll
          ++ playwrightPackages
          ++ [ bootstrapReleaseTools ];

        env = rustPackages.env // desktopPackages.env // android.env // {
          LD_LIBRARY_PATH = "${playwrightLibPath}:${rustPackages.libPath}";
        };

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
