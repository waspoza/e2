{
  description = "Search for text inside emails";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "e";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          doCheck = false; # speed thing up by avoiding second build

          meta = with pkgs.lib; {
            description = "Search for text inside emails";
            mainProgram = "e";
            license = licenses.bsd3;
            platforms = platforms.linux;
          };
        };
      }
    );
}
