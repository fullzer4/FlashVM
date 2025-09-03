{
  description = "flashVM – reproducible OCI embed + PyPI publishing via Nix";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
  let
    linuxSystems = [ "x86_64-linux" "aarch64-linux" ];
    forAll = nixpkgs.lib.genAttrs linuxSystems;
  in {
    formatter = forAll (system: (import nixpkgs { inherit system; }).nixfmt);

    packages = forAll (system:
      let
        pkgs = import nixpkgs { inherit system; };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "python-basic";
          tag  = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "python-basic-rootfs";
            paths = [ pkgs.coreutils pkgs.bash pkgs.python312Full ];
          };

          config = {
            Cmd = [ "python3" "-u" ];
            WorkingDir = "/";
          };
        };

        ociLayout = pkgs.stdenvNoCC.mkDerivation {
          pname = "flashvm-oci-layout";
          version = "0.1.0";
          nativeBuildInputs = [ pkgs.skopeo ];
          dontUnpack = true;
          installPhase = ''
            mkdir -p $out/oci
            # tmp/runtime para skopeo funcionar no sandbox
            TMP=${TMPDIR:-$PWD/tmp}
            mkdir -p "$TMP"
            export TMPDIR="$TMP"
            export XDG_RUNTIME_DIR="$TMP"
            # docker-archive:<tar> -> oci:<dir>:tag
            skopeo --tmpdir "$TMP" copy --insecure-policy docker-archive:${dockerImage} oci:$out/oci:python-basic
          '';
        };
      in {
        default    = ociLayout;
        oci-layout = ociLayout;
      }
    );

    devShells = forAll (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        default = pkgs.mkShell {
          packages = [
            pkgs.python312
            pkgs.uv
            pkgs.maturin
            pkgs.rustc
            pkgs.cargo
            pkgs.pkg-config
            pkgs.skopeo
            pkgs.buildah
            pkgs.podman
          ];
          shellHook = ''
            echo ">> flashVM devshell — Python: $(python3 -V)"
            echo "Use: nix run .#vendor-oci && nix run .#develop (in-tree) ou nix run .#build (wheels)"
          '';
        };
      }
    );

    apps = forAll (system:
      let
        pkgs = import nixpkgs { inherit system; };
        oci = self.packages.${system}.oci-layout;

        vendorScript = pkgs.writeShellScript "vendor-oci" ''
          set -euo pipefail
          dst="flashvm/data/oci"
          if [ -e "$dst" ]; then
            chmod -R u+w "$dst" 2>/dev/null || true
            rm -rf "$dst"
          fi
          mkdir -p "$dst"
          cp -R "${oci}/oci/." "$dst/"
          echo "✅ Vendored OCI layout into $dst"
        '';

        vendorDockerScript = pkgs.writeShellScript "vendor-oci-from-dockerfile" ''
          set -euo pipefail
          dst="flashvm/data/oci"
          if [ -e "$dst" ]; then
            chmod -R u+w "$dst" 2>/dev/null || true
            rm -rf "$dst"
          fi
          mkdir -p "$dst"
          img="localhost/flashvm-python-basic:latest"
          ${pkgs.buildah}/bin/buildah bud -t "$img" -f docker/Dockerfile.python-basic docker
          ${pkgs.skopeo}/bin/skopeo --insecure-policy copy containers-storage:"$img" oci:"$dst":python-basic
          echo "✅ Vendored OCI layout from docker/Dockerfile.python-basic into $dst"
        '';

        buildScript = pkgs.writeShellScript "build-flashvm" ''
          set -euo pipefail
          ${pkgs.maturin}/bin/maturin build
          wheel=$(ls -1t target/wheels/flashvm-*.whl | head -n1)
          if [ -z "${wheel:-}" ]; then
            echo "Could not find built wheel in target/wheels"; exit 1;
          fi
          echo "Extracting native extension from $wheel into flashvm/"
          WHEEL="$wheel" python3 - <<'PY'
import os, zipfile, pathlib, sys
wheel = os.environ['WHEEL']
zf = zipfile.ZipFile(wheel)
members = [m for m in zf.namelist() if m.startswith('flashvm/') and (m.endswith('.so') or m.endswith('.pyd'))]
if not members:
    print('No native extension found inside wheel')
    sys.exit(1)
target = pathlib.Path('flashvm')
target.mkdir(exist_ok=True)
for m in members:
    name = m.split('/')[-1]
    with zf.open(m) as src, open(target / name, 'wb') as dst:
        dst.write(src.read())
print(f"Extracted: {', '.join(members)}")
PY
        '';

        developScript = pkgs.writeShellScript "develop-flashvm" ''
          set -euo pipefail
          ${pkgs.maturin}/bin/maturin develop
          if ls flashvm/_core* 1>/dev/null 2>&1; then
            echo "✅ Found flashvm/_core*.so in repo; in-tree imports will work"
            exit 0
          fi
          py="$(command -v python3)"
          so="$($py - <<'PY'
import sysconfig, pathlib
paths = []
for key in ("platlib","purelib"):
    p = sysconfig.get_paths().get(key)
    if p:
        paths.append(p)
found = None
for base in paths:
    p = pathlib.Path(base)/"flashvm"
    for ext in (".so", ".pyd"):
        matches = list(p.glob(f"_core*{ext}"))
        if matches:
            found = matches[0]
            break
    if found:
        break
print(found if found else "")
PY
)"
          if [ -n "$so" ]; then
            mkdir -p flashvm
            ln -sf "$so" "flashvm/$(basename "$so")"
            echo "✅ Linked $(basename "$so") into flashvm/ for in-tree imports"
            exit 0
          fi
          echo "Did not locate _core in site-packages; building a wheel to extract"
          ${pkgs.maturin}/bin/maturin build
          wheel=$(ls -1t target/wheels/flashvm-*.whl | head -n1)
          if [ -z "${wheel:-}" ]; then
            echo "Could not find built wheel in target/wheels"; exit 1
          fi
          WHEEL="$wheel" python3 - <<'PY'
import os, zipfile, pathlib
wheel = os.environ['WHEEL']
zf = zipfile.ZipFile(wheel)
members = [m for m in zf.namelist() if m.startswith('flashvm/') and (m.endswith('.so') or m.endswith('.pyd'))]
if not members:
    raise SystemExit('No native extension found inside wheel')
target = pathlib.Path('flashvm')
target.mkdir(exist_ok=True)
for m in members:
    name = m.split('/')[-1]
    with zf.open(m) as src, open(target / name, 'wb') as dst:
        dst.write(src.read())
print(f"✅ Extracted {', '.join(members)} into flashvm/")
PY
        '';

        publishScript = pkgs.writeShellScript "publish-flashvm" ''
          set -euo pipefail
          test -n "''${UV_PUBLISH_TOKEN-}" || { echo "UV_PUBLISH_TOKEN not set"; exit 1; }
          # Build only wheels to avoid sdist (git HEAD) issues
          rm -rf dist
          ${pkgs.maturin}/bin/maturin build --release -o dist
          if ! ls dist/*.whl 1>/dev/null 2>&1; then
            echo "No wheels found in dist/"; exit 1;
          fi
          # Publish wheel(s)
          ${pkgs.uv}/bin/uv publish --token "$UV_PUBLISH_TOKEN" dist/*.whl
        '';
      in {
        vendor-oci        = { type = "app"; program = "${vendorScript}"; };
        vendor-oci-docker = { type = "app"; program = "${vendorDockerScript}"; };
        build             = { type = "app"; program = "${buildScript}"; };
        develop           = { type = "app"; program = "${developScript}"; };
        publish           = { type = "app"; program = "${publishScript}"; };
      }
    );
  };
}
