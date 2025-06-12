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
  ];

  # Configura corretamente LD_LIBRARY_PATH para Rust rodar com bibliotecas nativas
  shellHook = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
  '';
}

