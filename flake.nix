{
  description = "Wayland keypress display using GTK4 and libinput";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {inherit system;};
        toolchain = pkgs.rustPlatform;

        runtimeDeps = with pkgs; [
          gtk4
          gtk4-layer-shell
          libinput
          wayland
          wayland-protocols
          dbus
        ];

        buildInputs = with pkgs; [
          gtk4
          gtk4-layer-shell
          libinput
          wayland
          wayland-protocols
          pkg-config
          dbus
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          wrapGAppsHook4
        ];
      in rec
      {
        packages.default = toolchain.buildRustPackage {
          pname = "keystroke";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          inherit buildInputs nativeBuildInputs;

          preFixup = ''
            gappsWrapperArgs+=(
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeDeps}"
            )
          '';
        };

        apps.default = utils.lib.mkApp {drv = packages.default;};

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              (with toolchain; [
                cargo
                rustc
                rustLibSrc
              ])
              clippy
              rustfmt
              rust-analyzer
            ]
            ++ buildInputs ++ nativeBuildInputs;

          RUST_SRC_PATH = "${toolchain.rustLibSrc}";

          PKG_CONFIG_PATH = "${pkgs.lib.makeSearchPath "lib/pkgconfig" buildInputs}";

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath runtimeDeps}";

          XDG_DATA_DIRS = "${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:$XDG_DATA_DIRS";
        };
      }
    );
}
