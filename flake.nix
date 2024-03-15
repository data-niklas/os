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

          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = "1.75.0";
            packageFun = import ./Cargo.nix;
            packageOverrides = pkgs:
              pkgs.rustBuilder.overrides.all
              ++ [
                # parentheses disambiguate each makeOverride call as a single list element
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gtk4-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.gtk4
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gtk4-layer-shell-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.gtk4-layer-shell
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "glib-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.glib
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gobject-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.glib
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "graphene-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.graphene
                        pkgs.gobject-introspection
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "cairo-sys-rs";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.cairo
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gio-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.glib
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "pango-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.pango
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gdk-pixbuf-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.gdk-pixbuf
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gdk4-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.gtk4
                        pkgs.pkg-config
                      ];
                  };
                })
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "gsk4-sys";
                  overrideAttrs = drv: {
                    nativeBuildInputs =
                      drv.nativeBuildInputs
                      ++ [
                        pkgs.gtk4
                        pkgs.pkg-config
                      ];
                  };
                })
              ];
          };
          workspaceShell = rustPkgs.workspaceShell {};
        in rec {
          packages = {
            # replace hello-world with your package name
            os = rustPkgs.workspace.os {};
            default = packages.os;
          };
          devShells = {
            # nix develop
            default = workspaceShell;
          };
        }
      );
}
