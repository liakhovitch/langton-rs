with (import <nixpkgs> {});
let
  inputs = [
    rustc
    cargo
    clang
    openssl
    pkgconfig
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libudev0-shim
    libudev
    vulkan-headers
    vulkan-loader
    vulkan-tools
  ];

  lib_path = "${lib.makeLibraryPath inputs}";
in mkShell {

  name = "rust-env";

  buildInputs = inputs;
  LD_LIBRARY_PATH = lib_path;
}
