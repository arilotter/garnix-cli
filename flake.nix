{
  description = "Garnix CLI - Run Garnix-compatible nix builds locally";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      treefmt-nix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };

        treefmtEval = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            rustfmt.enable = true;
            yamlfmt.enable = true;
          };
        };

        garnix-cli-config = {
          pname = "garnix-cli";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs =
            with pkgs;
            [
              openssl
              libgit2
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

          meta = with pkgs.lib; {
            description = "run the same builds garnix does";
            homepage = "https://github.com/arilotter/garnix-cli";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        garnix-cli = pkgs.rustPlatform.buildRustPackage garnix-cli-config;

      in
      {
        formatter = treefmtEval.config.build.wrapper;

        packages = {
          default = garnix-cli;
          garnix-cli = garnix-cli;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ garnix-cli ];
          buildInputs =
            with pkgs;
            [
              nix-output-monitor

              # dev tools
              cargo-watch
              rust-analyzer
            ]
            ++ (builtins.attrValues treefmtEval.config.build.programs);

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        apps.default = flake-utils.lib.mkApp {
          drv = garnix-cli;
          name = "garnix";
        };

        checks = {
          garnix-cli = garnix-cli;

          cargo-test = pkgs.rustPlatform.buildRustPackage {
            pname = "garnix-cli-tests";
            inherit (garnix-cli-config)
              version
              src
              cargoLock
              nativeBuildInputs
              buildInputs
              ;

            buildPhase = ''
              cargo test
            '';
            installPhase = ''
              mkdir -p $out
              echo "tests passed" > $out/test-results
            '';
          };

          treefmt = treefmtEval.config.build.check self;
        };
      }
    );
}
