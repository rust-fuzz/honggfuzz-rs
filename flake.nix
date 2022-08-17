{
  description = "honggfuzz-rs: Fuzz your Rust code with Google-developed Honggfuzz !";

  inputs = {
    nixpkgs.url = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";
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
        ];
      };

      packages.default = packages.honggfuzz-rs;
    });
}
