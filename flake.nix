{
  description = "flashvm flake (packages + devShell)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          let pkgs = import nixpkgs { inherit system; }; in f pkgs system);
    in {
      packages = forAllSystems (pkgs: system: {
        default = pkgs.callPackage ./package.nix { inherit pkgs; };
      });

      devShells = forAllSystems (pkgs: system: {
        default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.maturin
            pkgs.python311
            pkgs.pkg-config
            pkgs.openssl
          ];
          shellHook = ''
            export PYO3_PYTHON=${pkgs.python311}/bin/python3
          '';
        };
      });
    };
}
