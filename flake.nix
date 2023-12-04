{
  description = "A flake to support reproducible development of t3rn";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    noir-lang.url = "github:noir-lang/noir/v0.19.2";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, noir-lang, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { 
          inherit system overlays; 
        };
        rustVersion = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
      in {
        stdenv = pkgs.clangStdenv;
        devShell = pkgs.mkShell {
          LIBCLANG_PATH = pkgs.libclang.lib + "/lib/";
          inputsFrom = [
            noir-lang
            # noir_wasm
            # noirc_abi_wasm
            # acvm_js
          ];

          nativeBuildInputs = with pkgs; [ 
            bashInteractive
            taplo
            cmake
            openssl
            pkg-config
            # clang
            llvmPackages_11.bintools 
            llvmPackages_11.libclang 
            protobuf

            yarn
            nodejs-18_x

            noir-lang.packages.${system}.nargo

          ];
          buildInputs = with pkgs; [ 
              (rustVersion.override { extensions = [ "rust-src" ]; }) 
          ];
          
        };
  });
}
