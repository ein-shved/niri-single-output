{
  description = ''
    Simple utility to control niri outputs withing single-output scheme
  '';

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        niri-single-output = pkgs.callPackage ./. {};
      in
      {
        packages = {
          inherit niri-single-output;
          default = niri-single-output;
        };
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
