{ pkgs
, python ? pkgs.python311
}:

let
  py = python.pkgs;
in
py.buildPythonPackage {
  pname = "flashvm";
  version = "0.1.0";

  format = "pyproject";
  src = ./.;

  nativeBuildInputs = [
    pkgs.maturin
    pkgs.rustc
    pkgs.cargo
    pkgs.pkg-config
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.openssl ];

  doCheck = false;
}
