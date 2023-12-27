{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              fenix.packages.${system}.minimal.rustc
              # fenix.packages.${system}.minimal.rustfmt
              # fenix.packages.${system}.minimal.clippy
              # fenix.packages.${system}.minimal.cargo

             
              
              pre-commit
              libclang
              libv4l
              v4l-utils
              linuxHeaders
              xorg.libX11
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS =
              "-I${linuxHeaders}/include -I${glibc.dev}/include";
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [
              v4l-utils
              libv4l
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              libglvnd
              libusb1
            ]}";
          };
      });
}
