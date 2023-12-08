{
  description = "A flake to support reproducible";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    noir-lang.url = "github:noir-lang/noir/v0.19.4";
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
        stdenv = pkgs.llvmPackages.libcxxStdenv;
        devShell = pkgs.mkShell {
          LIBCLANG_PATH = pkgs.llvmPackages.libclang.lib + "/lib/";
          LD_LIBRARY_PATH = "${pkgs.llvmPackages.libclang.lib}/lib:${pkgs.llvmPackages.libcxx}/lib:${pkgs.llvmPackages.libcxxabi}/lib:/usr/local/lib";


          inputsFrom = [
            noir-lang
          ];

          nativeBuildInputs = with pkgs; [ 
            bashInteractive
            taplo
            cmake
            openssl
            pkg-config
            # clang
            llvmPackages.bintools 
            llvmPackages.libclang 
            llvmPackages.libcxx
            llvmPackages.libcxxabi
            protobuf
            leptonica

            pkg-config
            #yarn
            #nodejs-18_x

            noir-lang.packages.${system}.nargo

          ];
          buildInputs = with pkgs; [ 
              (rustVersion.override { extensions = [ "rust-src" ]; }) 
          ];
          
        };
  });
}
