{
  lib,
  rustPlatform,
  cmake,
  ffmpeg,
}:
let
  cargoToml = (lib.importTOML ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = ./..;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    cmake
  ];

  buildInputs = [
    ffmpeg
  ];

  meta = with lib; {
    description = "A lightweight TUI music player for local files";
    homepage = "https://github.com/Jaxx497/NoctaVox";
    license = licenses.mit;
    mainProgram = "vox";
  };
}
