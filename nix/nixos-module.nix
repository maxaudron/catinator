{ lib, config, pkgs, ... }:

let
  cfg = config.services.catinator;
  toml = pkgs.formats.toml { };
  configFile = toml.generate "config.toml" cfg.settings;
in

with lib;
{
  options = {
    services.catinator = {
      package = mkOption {
        defaultText = lib.literalMD "`packages.default` from the catinator flake";
      };

      extraEnvironment = mkOption {
        type = types.attrsOf types.str;
        description = "Extra environment variables to pass to the Garage server.";
        default = { };
        example = { RUST_BACKTRACE = "yes"; };
      };

      environmentFile = mkOption {
        type = types.nullOr types.path;
        description = "File containing environment variables to be passed to the Garage server.";
        default = null;
      };

      logLevel = mkOption {
        type = types.enum ([ "error" "warn" "info" "debug" "trace" ]);
        default = "info";
        example = "debug";
      };

      settings = mkOption {
        type = types.submodule {
          freeformType = toml.type;
        };
      };
    };
  };

  config = {
    services.catinator.settings = {
      default = {
        user = {
          username = lib.mkDefault "catinator";
          realname = lib.mkDefault "moaw";
        };

        server = {
          hostname = lib.mkDefault "";
          port = lib.mkDefault 6697;
          tls = lib.mkDefault true;
          sasl = lib.mkDefault true;
        };

        settings = {
          prefix = lib.mkDefault ":";
        };
      };

      release = {
        user = {
          nickname = lib.mkDefault "\\__{^-_-^}";
        };

        server = {
         channels = lib.mkDefault "";
        };
      };
    };

    systemd.services.catinator = {
      description = "Catinator IRC Bot";
      after = [ "network.target" "network-online.target" ];
      wants = [ "network.target" "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      restartTriggers = [ configFile ] ++ (lib.optional (cfg.environmentFile != null) cfg.environmentFile);
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/catinator";

        DynamicUser = lib.mkDefault true;
        ProtectHome = true;
        NoNewPrivileges = true;
        EnvironmentFile = lib.optional (cfg.environmentFile != null) cfg.environmentFile;
      };
      environment = {
        RUST_LOG = lib.mkDefault "catinator=${cfg.logLevel}";
        CATINATOR_CONFIG = configFile;
      } // cfg.extraEnvironment;
    };
  };
}
