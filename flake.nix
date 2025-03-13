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

    crane = { # eventually, use dream2nix when it's more stable
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, nixpkgs, fenix, crane, ...}: flake-parts.lib.mkFlake { inherit inputs; } {
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
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          hardeningDisable = [ "fortify" ];
        };
      };

      devenv.shells.default = {
        packages = with pkgs; [
          libbfd
          bintools-unwrapped
          libunwind
        ];

        env = {
          NIX_HARDENING_ENABLE = "";
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
