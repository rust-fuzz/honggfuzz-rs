{
  description = "honggfuzz-rs: Fuzz your Rust code with Google-developed Honggfuzz !";

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  inputs = {
    # nixpkgs = {
    #   type = "indirect"; # take it from the registry
    #   id   = "nixpkgs";
    # };

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    #nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    #mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";

    rust-overlay = {
      url = "github:oxalica/rust-overlay/29a57fd94e9f384597222fb3301466a112a8c200"; # https://github.com/cachix/devenv/pull/2558
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = { # eventually, use dream2nix when it's more stable
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, nixpkgs, systems, fenix, crane, ...}: flake-parts.lib.mkFlake { inherit inputs; } {
    imports = [
      inputs.devenv.flakeModule
    ];
    systems = nixpkgs.lib.systems.flakeExposed;

    perSystem = {system, pkgs, self', ...}: let
      pkgs-fenix = import nixpkgs {
        inherit system;
        overlays = [ fenix.overlays.default ];
      };
    in {
      packages = let
        #craneLib = crane.mkLib pkgs;
        craneLib = (crane.mkLib pkgs-fenix).overrideToolchain (p: p.fenix.minimal.toolchain); # rust nightly
      in rec {
        default = honggfuzz-rs;
        honggfuzz-rs = craneLib.buildPackage {
          #stdenv = pkgs.clangStdenv;
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          #hardeningDisable = [ "fortify" ];
        };
      };

      devenv.shells.default = rec {
        #stdenv = pkgs.clangStdenv;
        #stdenv = pkgs.gcc15Stdenv;
        packages = with pkgs; [
          libbfd
          libunwind
        ] ++ lib.optional stdenv.cc.isClang pkgsStatic.libblocksruntime;

        env = {
          NIX_HARDENING_ENABLE = ""; # to disable fortify flag
        };

        languages = {
          rust = {
            enable = true;
            channel = "nightly";
          };
        };
      };
    };
  };
}
