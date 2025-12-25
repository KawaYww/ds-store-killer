{
  description = "Kill .DS_Store files on macOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { lib, system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.minimal;

          mkPackage = targetPkgs:
            targetPkgs.rustPlatform.buildRustPackage {
              pname = "dsk";
              version = "0.1.0";
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              doCheck = false;
              enableParallelBuilding = true;

              meta = {
                description = "Kill .DS_Store files on macOS";
                homepage = "https://github.com/kawayww/ds-store-killer";
                license = lib.licenses.mit;
                mainProgram = "dsk";
              };
            };
        in
        {
          packages = {
            default = mkPackage pkgs;
            aarch64-darwin = mkPackage pkgs.pkgsCross.aarch64-darwin;
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = [ rustToolchain pkgs.rust-analyzer ];
            RUST_BACKTRACE = 1;
          };
        };
    };
}
