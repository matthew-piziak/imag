{ pkgs ? (import <nixpkgs> {}) }:

let
  env = with pkgs.rustStable; [
    rustc
    cargo
  ];

  deps = with pkgs; [
    dbus
    pkgconfig
  ];
in

pkgs.stdenv.mkDerivation rec {
    name = "imag";
    src = ./.;
    version = "0.0.0";

    buildInputs = [ env deps ];

}

