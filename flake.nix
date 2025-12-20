{
  description = "Rust challenges workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.rustc
          pkgs.cargo
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
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.wayland
            pkgs.vulkan-loader
          ];
        };
      };
    };
}
