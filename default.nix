{
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "niri-single-output";
  version = "0.1.0";
  src =
    builtins.path {
      filter = (
        path: type:
        let
          bn = baseNameOf path;
        in
        bn != "flake.nix" && bn != "flake.lock" && bn != "default.nix"
      );
      path = ./.;
    };
  cargoLock.lockFile = ./Cargo.lock;
}
