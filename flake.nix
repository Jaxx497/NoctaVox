{
  description = "A lightweight TUI music player for local files";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
        { pkgs, ... }:
        {
          devShells.default = pkgs.mkShell {
            strictDeps = true;
            nativeBuildInputs = [
              pkgs.cargo
              pkgs.rustc
              pkgs.pkg-config
              pkgs.cmake
            ];

            buildInputs = [
              pkgs.ffmpeg
            ];
          };

          packages = rec {
            noctavox = pkgs.callPackage ./nix/package.nix { };
            default = noctavox;
          };
        };
      flake = {
      };
    };
}
