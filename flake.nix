{
  description = "flashVM";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
  let
    systems = [ "x86_64-linux" "aarch64-linux" ];
    forAll = nixpkgs.lib.genAttrs systems;
  in {
    devShells = forAll (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        default = pkgs.mkShell {
          packages = [
            pkgs.python312 pkgs.maturin pkgs.uv
            pkgs.rustc pkgs.cargo pkgs.pkg-config
            pkgs.docker pkgs.docker-buildx pkgs.gnutar pkgs.coreutils pkgs.jq
            pkgs.umoci pkgs.skopeo
            pkgs.erofs-utils pkgs.squashfsTools pkgs.zstd pkgs.lz4
            pkgs.busybox pkgs.cpio pkgs.e2fsprogs pkgs.cloud-hypervisor
          ];
          shellHook = ''echo ">> flashVM dev shell ativa"'';
        };
      }
    );
  };
}
