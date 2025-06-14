{ pkgs ? import <nixpkgs> { }, lib ? pkgs.lib }:

pkgs.mkShell rec {
  name = "rust-env";

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = with pkgs; [
    rustc
    cargo
    clang
    openssl
    glib
    atk
    gtk3                 # inclui gdk, gtk e dependências como gdk-pixbuf, pango, cairo
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
  ];

  # Configura corretamente LD_LIBRARY_PATH para Rust rodar com bibliotecas nativas
  shellHook = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
    export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
  '';
}

