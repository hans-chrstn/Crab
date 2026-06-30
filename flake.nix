{
  description = "Test";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    systems = [
      "aarch64-linux"
      "i686-linux"
      "x86_64-linux"
      "aarch64-darwin"
      "x86_64-darwin"
    ];
    forAllSystems = f:
      nixpkgs.lib.genAttrs systems (system:
        f {
          pkgs = let
            overlays = [(import rust-overlay)];
          in
            import nixpkgs {
              inherit system overlays;
              config = {allowUnfree = true;};
            };
        });
  in {
    devShells = forAllSystems ({pkgs}: {
      default = pkgs.mkShell rec {
        packages = with pkgs; [
          pkg-config
          cmake
          ninja
          hidapi
          dbus
          qt6.qtbase
          qt6.qtdeclarative
          qt6.qtwayland
          (rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rust-analyzer"];
          })
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
      };
    });
  };
}
