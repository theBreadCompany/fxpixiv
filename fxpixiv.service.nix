# fxpixiv.service.nix
{ config, pkgs, lib, ... }:

let
  inherit (lib) mkOption mkIf types;
  fxpixivApp = import ./default.nix { inherit pkgs lib; rustPlatform = pkgs.rustPlatform; };
in
{
  options.services.fxpixiv = {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the fxpixiv embed helper service.";
    };

    port = mkOption {
      type = types.int;
      description = "Port on which the fxpixiv embed helper listens.";
    };

    refreshToken = mkOption {
      type = types.str;
      default = "";
      description = "Allows access to the Pixiv API.";
    };
  };

  config = mkIf config.services.fxpixiv.enable {
    systemd.services.fxpixiv = {
      description = "fxpixiv embed helper service";
      wantedBy = [ "multi-user.target" ];

      # Command to start the Rocket application
      serviceConfig.ExecStart = "${fxpixivApp}/bin/fxpixiv";

      # Set the port environment variable
      serviceConfig.Environment = [ 
        "ROCKET_ENV=release"
        "ROCKET_PORT=${toString config.services.fxpixiv.port}"
        "PIXIV_REFRESH_TOKEN=${toString config.services.fxpixiv.refreshToken}"
      ];

      # Restart on failure
      serviceConfig.WorkingDirectory = "/var/lib/fxpixiv";
      serviceConfig.ReadWritePaths = [ "/var/lib/fxpixiv" ];
      serviceConfig.Restart = "on-failure";
      serviceConfig.User = "fxpixiv";
      serviceConfig.Group = "fxpixiv";

      # Ensure the service starts after the network is up
      after = [ "network.target" ];
    };

    systemd.tmpfiles.rules = [ 
      "d /var/lib/fxpixiv 0755 fxpixiv fxpixiv" 
      ''f /var/lib/Rocket.toml 0644 fxpixiv fxpixiv - <<EOF
          [release]
          log_level = "critical"
          
          [default.databases.pixiv]
          url = "sqlite:fxpixiv.db"
	  EOF
      ''
    ];

    users.users.fxpixiv = {
      isSystemUser = true;
      home = "/var/lib/fxpixiv";
      group = "fxpixiv";
    };
    users.groups.fxpixiv = {};
  };
}
