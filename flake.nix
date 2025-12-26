{
  description = "Kill .DS_Store files on macOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # macOS only
      systems = [ "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      pkgsFor = system: import nixpkgs { inherit system; };

      mkPackage = pkgs: pkgs.rustPlatform.buildRustPackage {
        pname = "dsk";
        version = "0.2.1";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        doCheck = false;

        meta = {
          description = "Kill .DS_Store files on macOS";
          homepage = "https://github.com/pwnwriter/dsk";
          license = pkgs.lib.licenses.mit;
          mainProgram = "dsk";
          platforms = pkgs.lib.platforms.darwin;
        };
      };
    in
    {
      packages = forAllSystems (system: {
        default = mkPackage (pkgsFor system);
      });

      devShells = forAllSystems (system:
        let pkgs = pkgsFor system; in {
          default = pkgs.mkShell {
            nativeBuildInputs = [ pkgs.rustc pkgs.cargo pkgs.rust-analyzer ];
            RUST_BACKTRACE = 1;
          };
        }
      );

      # nix-darwin module
      darwinModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.dsk;
          package = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in
        {
          options.services.dsk = {
            enable = lib.mkEnableOption "dsk - .DS_Store killer daemon";

            paths = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = [ "~" ];
              description = "Directories to watch for .DS_Store files";
              example = [ "~/Desktop" "~/Projects" ];
            };

            notify = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Send macOS notifications on delete";
            };

            force = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Force delete git-tracked .DS_Store files (DANGER)";
            };

            exclude = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = [];
              description = "Patterns to exclude from watching";
              example = [ "node_modules" ".git" ];
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ package ];

            launchd.user.agents.dsk = {
              serviceConfig = {
                Label = "com.dsk.watcher";
                ProgramArguments =
                  [ "${package}/bin/dsk" "watch" ]
                  ++ lib.optionals cfg.notify [ "--notify" ]
                  ++ lib.optionals cfg.force [ "--force" ]
                  ++ lib.concatMap (e: [ "-e" e ]) cfg.exclude
                  ++ cfg.paths;
                RunAtLoad = true;
                KeepAlive = true;
                StandardOutPath = "/tmp/dsk.out.log";
                StandardErrorPath = "/tmp/dsk.err.log";
              } // lib.optionalAttrs cfg.notify {
                LimitLoadToSessionType = "Aqua";
              };
            };
          };
        };
    };
}
