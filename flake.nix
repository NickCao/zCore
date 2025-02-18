{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs; {
        devShells.default = mkShell {
          nativeBuildInputs = [
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            (python3.withPackages (ps: with ps; [ termcolor ]))
            wget
            qemu
            cargo-binutils
            rust-analyzer
          ];
        };
      }
    );
}
