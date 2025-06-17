{ pkgs ? import <nixpkgs> {}, lib ? pkgs.lib }:
let
    overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
    libPath = with pkgs; lib.makeLibraryPath [
        # load external libraries that you need in your rust project here
    ];
    pkgs-cross-mingw = import pkgs.path {
        crossSystem = {
            config = "x86_64-w64-mingw32";
        };
    };
    mingw_w64_cc = pkgs-cross-mingw.stdenv.cc;
    mingw_w64 = pkgs-cross-mingw.windows.mingw_w64;
    mingw_w64_pthreads_w_static = pkgs-cross-mingw.windows.mingw_w64_pthreads.overrideAttrs (oldAttrs: {
        # TODO: Remove once / if changed successfully upstreamed.
        configureFlags = (oldAttrs.configureFlags or []) ++ [
            # Rustc require 'libpthread.a' when targeting 'x86_64-pc-windows-gnu'.
            # Enabling this makes it work out of the box instead of failing.
            "--enable-static"
        ];
    });
in
    pkgs.mkShell rec {
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs; [
            clang
            # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
            openssl
            mingw_w64_cc
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

            llvmPackages.bintools
            rustup
        ];
        RUSTC_VERSION = overrides.toolchain.channel;
        # https://github.com/rust-lang/rust-bindgen#environment-variables
        LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
        shellHook = ''
      export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
      export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
      export LD_LIBRARY_PATH=${lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
      export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
        '';
        # Add precompiled library to rustc search path
        RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
            mingw_w64
            mingw_w64_pthreads_w_static
            # add libraries here (e.g. pkgs.libvmi)
        ]);
        LD_LIBRARY_PATH = libPath;
        # Add glibc, clang, glib, and other headers to bindgen search path
        BINDGEN_EXTRA_CLANG_ARGS =
            # Includes normal include path
            (builtins.map (a: ''-I"${a}/include"'') [
                # add dev libraries here (e.g. pkgs.libvmi.dev)
                pkgs.glibc.dev
            ])
            # Includes with special directory paths
            ++ [
                ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
                ''-I"${pkgs.glib.dev}/include/glib-2.0"''
                ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];
    }
