{
  rustPlatform,
  cargo-toml ? fromTOML (builtins.readFile ./Cargo.toml),
}:
rustPlatform.buildRustPackage {
  pname = cargo-toml.package.name;
  version = cargo-toml.package.version;

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;
}
