{
  description = "Rust challenges workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            })
            pkgs.go
            pkgs.gcc
            pkgs.kcl
          ]
          # gpui apps (e.g. conways) need the wayland graphics stack on Linux
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.glib
            pkgs.pango
            pkgs.libsoup_3
            pkgs.webkitgtk_4_1
            pkgs.libxkbcommon
            pkgs.vulkan-loader
            pkgs.wayland
          ];

          env = {
            RUST_BACKTRACE = "1";
          }
          // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.vulkan-loader
            ];
          };
        };
      }
    );
}
