{
  description = "honggfuzz-rs: Fuzz your Rust code with Google-developed Honggfuzz !";

  inputs = {
    nixpkgs.url = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = inputs: inputs.flake-utils.lib.eachDefaultSystem ( system: let
      pkgs = inputs.nixpkgs.legacyPackages.${system};
    in rec {
      packages.honggfuzz-rs = pkgs.rustPlatform.buildRustPackage rec {
        pname = "honggfuzz-rs";
        version = "git";

        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };

      devShells.default = pkgs.mkShell rec {
        name = "honggfuzz-rs-devshell";

        buildInputs = with pkgs; [
          libbfd
          bintools-unwrapped
          libunwind
        ];

        nativeBuildInputs = with pkgs; [
          packages.honggfuzz-rs
          inputs.rust-overlay.packages.${system}.rust # rustc from nixpkgs fails to compile hfuzz targets
        ];
      };

      packages.default = packages.honggfuzz-rs;
    });
}
