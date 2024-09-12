# pxiv.service.nix
{ config, pkgs, lib, ... }:

let
  inherit (lib) mkOption mkIf types;
  pxivApp = import ./default.nix { inherit pkgs lib; rustPlatform = pkgs.rustPlatform; };
in
{
  options.services.pxiv = {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the pxiv embed helper service.";
    };

    port = mkOption {
      type = types.int;
      default = 8000;
      description = "Port on which the pxiv embed helper listens.";
    };
  };

  config = mkIf config.services.pxiv.enable {
    systemd.services.pxiv = {
      description = "pxiv embed helper service";
      wantedBy = [ "multi-user.target" ];

      # Command to start the Rocket application
      serviceConfig.ExecStart = "${pxivApp}/bin/pxiv";

      # Set the port environment variable
      serviceConfig.Environment = [ "ROCKET_PORT=${toString config.services.pxiv.port}" ];

      # Restart on failure
      serviceConfig.Restart = "always";

      # Ensure the service starts after the network is up
      after = [ "network.target" ];
    };
  };
}
