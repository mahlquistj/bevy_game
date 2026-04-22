{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    bevy-cli = {
      url = "github:TheBevyFlock/bevy_cli";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    bevy-cli,
    ...
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        libs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXrandr
          xorg.libXcursor
          xorg.libXi
          libxkbcommon
          wayland
        ];
        libPath = pkgs.lib.makeLibraryPath libs;
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;
      in
        with pkgs; {
          devShells.default = mkShell {
            RUST_SRC_PATH = "${rust.packages.stable.rustPlatform.rustLibSrc}";
            LD_LIBRARY_PATH = libPath;

            buildInputs = [rustToolchain];
            packages =
              [
                bacon
                cargo-nextest
                cargo-generate
                pkg-config
                bevy-cli.packages.${system}.default
              ]
              ++ libs;
          };
        }
    );
}
