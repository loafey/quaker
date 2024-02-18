{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { flake-utils, nixpkgs, naersk, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = with fenix.packages.${system};  combine [
          minimal.cargo
          minimal.rustc
          latest.clippy
          latest.rust-src
          latest.rustfmt
        ];
        min-pkgs = with pkgs; [
          pkg-config
          openssl
          gcc
          udev
          alsa-lib
          xorg.libX11
          xorg.libXi
          xorg.libXcursor
          xorg.libXrandr
          vulkan-tools
          vulkan-headers
          vulkan-loader
          vulkan-validation-layers

          libxkbcommon
          wayland
        ];
      in
      {
        defaultPackage = (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [ ] ++ min-pkgs;
        };

        devShell = (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage {
          src = ./.;
          dontPatchELF = true;
          nativeBuildInputs = with pkgs; [ ] ++ min-pkgs;
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
              pkgs.lib.makeLibraryPath [
                pkgs.udev
                pkgs.alsaLib
                pkgs.vulkan-loader
                pkgs.libxkbcommon
              ]
            }"'';
        };
      });
}
