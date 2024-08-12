{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nci.url = "github:yusdacra/nix-cargo-integration";
    nci.inputs.nixpkgs.follows = "nixpkgs";
    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs =
    inputs@{ self, parts, nci, ... }:
    parts.lib.mkFlake { inherit inputs; } ({moduleWithSystem, withSystem, ...}: {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      imports = [ nci.flakeModule ];

      flake = {
        # nixosModules.default = flake-parts-lib.importApply ./nix/nixos-module.nix { localFlake = self; inherit withSystem; };
        nixosModules.default = moduleWithSystem (
          perSystem@{ config }:  # NOTE: only explicit params will be in perSystem
          nixos@{ ... }:
          {
            services.catinator.package = perSystem.config.packages.default;
            imports = [ ./nix/nixos-module.nix ];
          }
        );
      };

      perSystem =
        {
          pkgs,
          config,
          lib,
          ...
        }:
        let
          # shorthand for accessing this crate's outputs
          # you can access crate outputs under `config.nci.outputs.<crate name>` (see documentation)
          crateOutputs = config.nci.outputs."catinator";
        in
        {
          nci = {
            projects."catinator".path = ./.;
            crates."catinator" =
              let
                mkDerivation = {
                  nativeBuildInputs = [ pkgs.file.dev ];
                };
              in
              {
                drvConfig = {
                  inherit mkDerivation;
                };
                depsDrvConfig = {
                  inherit mkDerivation;
                };
              };

            toolchainConfig = {
              channel = "stable";
              targets = [ "x86_64-unknown-linux-musl" ];
              components = [
                "rustfmt"
                "rust-src"
              ];
            };
          };

          devShells.default = crateOutputs.devShell;
          packages.default = crateOutputs.packages.release;
        };
    });
}
