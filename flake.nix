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
          inherit system;
        });
  in {
    packages = forAllSystems ({pkgs, ...}: {
      default = pkgs.rustPlatform.buildRustPackage rec {
        pname = "crab";
        version = "0.1.0";
        src = ./.;

        cargoBuildFlags = ["-p" "daemon"];

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs; [
          hidapi
          dbus
          udev
        ];
      };
    });

    devShells = forAllSystems ({pkgs, ...}: {
      default = pkgs.mkShell rec {
        packages = with pkgs; [
          usbutils
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

    nixosModules.default = {
      config,
      lib,
      pkgs,
      ...
    }:
      with lib; let
        cfg = config.services.crab;
      in {
        options.services.crab = {
          enable = mkEnableOption "Crab Daemon";
          package = mkOption {
            type = types.package;
            default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
            description = "The package to use for the Crab daemon.";
          };
        };

        config = mkIf cfg.enable {
          services.udev.extraRules = ''
            SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", ATTRS{idProduct}=="c548", MODE="0660", GROUP="input"
          '';

          systemd.services.crab = {
            description = "Crab Daemon";
            wantedBy = ["multi-user.target"];
            serviceConfig = {
              ExecStart = "${cfg.package}/bin/daemon";
              Restart = "on-failure";

              DynamicUser = true;
              SupplementaryGroups = ["input"];

              ProtectSystem = "strict";
              ProtectHome = true;
              PrivateTmp = true;

              RuntimeDirectory = "crab";
              RuntimeDirectoryMode = "0755";
            };
          };
        };
      };
  };
}
