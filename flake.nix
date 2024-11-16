# start : bash   : nix run .#dev
# 或者 nix shell .#dev --command 'dev-shell'
{
  description = "A Nix-flake-based golang development environment .";

  inputs = {
    #nixpkgs.url = "github:NixOS/nixpkgs/23.11";
    #nixpkgs.url = "https://mirrors.ustc.edu.cn/nix-channels/nixpkgs-unstable/nixexprs.tar.xz";
    nixpkgs.url = "https://mirrors.ustc.edu.cn/nix-channels/nixos-24.05/nixexprs.tar.xz";
  };

  outputs = { self , nixpkgs ,... }: let
    # system should match the system you are running on
    system = "x86_64-linux";
    # system = "x86_64-darwin";
  in {
    packages."${system}".dev = let
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
      packages = with pkgs; [
          go_1_23
          gopls
          gcc
          upx
          jetbrains.goland # allowUnfree
          nushell
      ];
    in pkgs.runCommand "dev-shell" {
      # Dependencies that should exist in the runtime environment
      buildInputs = packages;
      # Dependencies that should only exist in the build environment
      nativeBuildInputs = [ pkgs.makeWrapper ];
    } ''
      mkdir -p $out/bin/
      ln -s ${pkgs.nushell}/bin/nu $out/bin/dev-shell
      wrapProgram $out/bin/dev-shell --set GOPROXY https://goproxy.cn,direct
      wrapProgram $out/bin/dev-shell --prefix PATH : ${pkgs.lib.makeBinPath packages}
    '';
  };
}
