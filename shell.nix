{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.llvmPackages_latest.llvm
    pkgs.llvmPackages_latest.clang
    pkgs.llvmPackages_latest.lld
    pkgs.llvmPackages_latest.bintools
  ];
}

