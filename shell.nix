{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.dbus
    pkgs.openssl
  ];
}
