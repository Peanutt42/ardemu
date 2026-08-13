{
  description = "ardemu";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        runtimeLibs = with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader

          # X11 fallback
          libx11
          libxcursor
          libxrandr
          libxi
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs =
            with pkgs;
            [
              pkg-config

              # Wayland
              wayland-protocols
              wayland-scanner

              # Vulkan / wgpu
              vulkan-headers
              vulkan-validation-layers
              vulkan-tools
            ]
            ++ runtimeLibs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
        };
      }
    );
}
