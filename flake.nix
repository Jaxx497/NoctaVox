{
  description = "A lightweight TUI music player for local files";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        { system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          noctavox = pkgs.callPackage ./nix/package.nix { inherit rustToolchain; };
        in
        {
          devShells.default = pkgs.mkShell {
            strictDeps = true;
            inputsFrom = [ noctavox ];
            nativeBuildInputs = [ rustToolchain ];
          };

          packages = {
            inherit noctavox;
            default = noctavox;
          };
        };
      flake = {
      };
    };
}
