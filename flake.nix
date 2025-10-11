{
  description = "WebGPU testing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
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
        devShells.default = with pkgs; mkShell.override {
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
        } rec {
          nativeBuildInputs = [
            (rust-bin.stable.latest.default.override { extensions = [ "rust-src" "rust-analyzer"]; })
            pkg-config
            cmake
            mold
          ];
          buildInputs = [
            # necessary for building wgpu in 3rd party packages (in most cases)
            libxkbcommon
            wayland xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi
            alsa-lib
            fontconfig freetype
            shaderc directx-shader-compiler

            libGL
            vulkan-headers vulkan-loader
            vulkan-tools vulkan-tools-lunarg
            vulkan-extension-layer
            vulkan-validation-layers # don't need them *strictly* but immensely helpful
          ];
          
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      }
    );
}
