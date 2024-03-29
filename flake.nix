{
  description = "rust wasm devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            just
            nodejs
            tailwindcss
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            leptosfmt
            trunk
          ];
        };
      }
    );
}


