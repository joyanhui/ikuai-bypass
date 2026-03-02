{
  description = "iKuai-bypass Nix flake development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              config.allowUnfree = true;
            }
          )
        );
    in
    {
      devShells = forAllSystems (
        pkgs:
        let
          devPackages = with pkgs; [
            go
            gopls
            golangci-lint
            delve
            gotools
            gcc
            upx
            python3
            nushell
            git
            curl
          ];
        in
        {
          default = pkgs.mkShell {
            packages = devPackages;
            shellHook = ''
              export GOPROXY=''${GOPROXY:-https://goproxy.cn,direct}
              export GOWORK=off
              echo "[ikuai-bypass] dev shell ready: $(go version)"
            '';
          };
        }
      );

      packages = forAllSystems (pkgs: {
        dev = pkgs.writeShellApplication {
          name = "dev";
          runtimeInputs = [ pkgs.nushell ];
          text = ''
            exec ${pkgs.nushell}/bin/nu
          '';
        };

        default = self.packages.${pkgs.stdenv.hostPlatform.system}.dev;
      });

      apps = forAllSystems (pkgs: {
        dev = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.dev}/bin/dev";
          meta = {
            description = "Launch Nushell development entrypoint";
          };
        };
        default = self.apps.${pkgs.stdenv.hostPlatform.system}.dev;
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
