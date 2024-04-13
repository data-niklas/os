{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [cargo2nix.overlays.default];
          };

          libPath = with pkgs;
            lib.makeLibraryPath [
              libGL
              libxkbcommon
              wayland
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
            ];

          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = "1.75.0";
            packageFun = import ./Cargo.nix;
            packageOverrides = pkgs:
              pkgs.rustBuilder.overrides.all
              ++ [
                # parentheses disambiguate each makeOverride call as a single list element
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "os";
                  overrideAttrs = drv: {
                    features = [
                      "linkding"
                      "duckduckgo"
                      "wayland"
                    ];
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.pkg-config
                        pkgs.makeWrapper
                      ];
                    buildInputs = with pkgs;
                      drv.buildInputs
                      ++ [
                        sqlite
                        xorg.libxcb
                        wayland
                      ];
                    postInstall = ''
                      wrapProgram $bin/bin/os --prefix LD_LIBRARY_PATH : "${libPath}"
                      wrapProgram $out/bin/os --prefix LD_LIBRARY_PATH : "${libPath}"
                    '';
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gtk4-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.gtk4
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gtk4-layer-shell-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.gtk4-layer-shell
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "glib-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.glib
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gobject-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.glib
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "graphene-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.graphene
                      pkgs.gobject-introspection
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "cairo-sys-rs";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.cairo
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gio-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.glib
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "pango-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.pango
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gdk-pixbuf-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.gdk-pixbuf
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gdk4-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.gtk4
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gsk4-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs = [
                      pkgs.pkg-config
                    ];
                    buildInputs = [
                      pkgs.gtk4
                    ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "atspi-proxies";
                  overrideAttrs = drv: {
                    dependencies =
                      drv.dependencies
                      ++ [
                        "zvariant"
                        (rustPkgs."registry+https://github.com/rust-lang/crates.io-index".zvariant."3.15.2" {}).out
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "accesskit_unix";
                  overrideAttrs = drv: {
                    dependencies =
                      drv.dependencies
                      ++ [
                        "zvariant"
                        (rustPkgs."registry+https://github.com/rust-lang/crates.io-index".zvariant."3.15.2" {}).out
                      ];
                  };
                })
              ];
          };
          workspaceShell = rustPkgs.workspaceShell {
            LD_LIBRARY_PATH = libPath;
          };
        in rec {
          packages = {
            # replace hello-world with your package name
            os =
              rustPkgs.workspace.os {};
            default = packages.os;
          };
          devShells = {
            # nix develop
            default = workspaceShell;
          };
        }
      );
}
