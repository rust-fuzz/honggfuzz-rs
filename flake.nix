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

  outputs = inputs:
    let
      pkgs = inputs.nixpkgs.legacyPackages.x86_64-linux;
    in rec {
      packages.x86_64-linux.honggfuzz-rs = pkgs.rustPlatform.buildRustPackage rec {
        pname = "honggfuzz-rs";
        version = "git";

        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };

      devShells.x86_64-linux.default = pkgs.mkShell rec {
        name = "honggfuzz-rs-devshell";

        buildInputs = with pkgs; [
          libbfd
          bintools-unwrapped
          libunwind
        ];

        nativeBuildInputs = with pkgs; [
          packages.x86_64-linux.honggfuzz-rs
          inputs.rust-overlay.packages.x86_64-linux.rust # rustc from nixpkgs fails to compile hfuzz targets
        ];
      };

      packages.x86_64-linux.default = packages.x86_64-linux.honggfuzz-rs;
    };
}
