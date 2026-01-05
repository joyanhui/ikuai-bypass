# start : bash   : nix run .#dev
# 或者 nix shell .#dev --command 'dev-shell'
{
  description = "A Nix-flake-based golang development environment .";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/25.11";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
    in
    {
      packages."${system}".dev =
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };
          packages = with pkgs; [
            go
            gopls
            gcc
            upx
            nushell
            fyne
          ];
        in
        pkgs.runCommand "dev-shell"
          {
            buildInputs = packages;
            nativeBuildInputs = [ pkgs.makeWrapper ];
          }
          ''
            mkdir -p $out/bin/
            ln -s ${pkgs.nushell}/bin/nu $out/bin/dev-shell
            wrapProgram $out/bin/dev-shell --set GOPROXY https://goproxy.cn,direct
            wrapProgram $out/bin/dev-shell --prefix PATH : ${pkgs.lib.makeBinPath packages}
          '';
    };
}
