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

    # this is the last version with glibc <= 2.37. Newer versions of glibc make honggfuzz fail to build, see https://github.com/google/honggfuzz/issues/518
    nixpkgs-glibc237.url = "github:NixOS/nixpkgs/nixos-23.05";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    }; 
  };

  outputs = inputs @ {flake-parts, nixpkgs, nixpkgs-glibc237, ...}: flake-parts.lib.mkFlake { inherit inputs; } {
    imports = [
      inputs.devenv.flakeModule
    ];
    systems = nixpkgs.lib.systems.flakeExposed;

    perSystem = {system, pkgs, self', ...}: let 
      pkgs-glibc237 = nixpkgs-glibc237.legacyPackages.${system};
    in {
      packages = rec {
        default = honggfuzz-rs;
        honggfuzz-rs = pkgs-glibc237.rustPlatform.buildRustPackage rec {
          pname = "honggfuzz-rs";
          version = "git";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      };

      devenv.shells.default = {
        stdenv = pkgs-glibc237.stdenv;

        packages = with pkgs; [
          libbfd
          bintools-unwrapped
          libunwind
        ];

        languages = {
          rust = {
            enable = true;
            channel = "stable";
          };
        };
      };
    };
  };
}
