{
  description = "ljp";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/f9f91492042402e41a4894d0e356da6c0b62c52";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            rust-bin.stable.latest.default
          ];
        };
      }
    );
}