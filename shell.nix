{ pkgs ? import <nixpkgs> { }, lib ? pkgs.lib }:

let
  # Lê o rust-toolchain.toml para fixar canal e componentes
  overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
in
pkgs.mkShell rec {
  name = "rustup-env";
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = with pkgs; [
    clang
    llvmPackages.bintools
    pkg-config

    # Dependências nativas que você já tinha
    openssl
    glib
    atk
    gtk3
    gdk-pixbuf
    pango
    cairo
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libGL
    wayland
    dbus
    gsettings-desktop-schemas
    gnome-settings-daemon

    rustup
  ];

  shellHook = ''
    export PATH=$PATH:${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:${RUSTUP_HOME:-/.rustup}/toolchains/${overrides.toolchain.channel}-x86_64-unknown-linux-gnu/bin

    export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH

    export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
  '';

  RUSTC_VERSION = overrides.toolchain.channel;
}

