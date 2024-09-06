# rocket-service.nix
{ config, pkgs, ... }:

let
  # Import the Rocket application package
  pxivApp = import ./default.nix { inherit pkgs; };
in
{
  options.services.rocket = {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the Rocket application service.";
    };

    port = mkOption {
      type = types.int;
      default = 8000;
      description = "Port on which the Rocket application listens.";
    };
  };

  config = mkIf config.services.rocket.enable {
    systemd.services.pxiv = {
      description = "pxiv embed helper service";
      wantedBy = [ "multi-user.target" ];

      # Command to start the Rocket application
      serviceConfig.ExecStart = "${pxivApp}/bin/pxiv";

      # Set the port environment variable
      serviceConfig.Environment = [ "ROCKET_PORT=${toString config.services.rocket.port}" ];

      # Restart on failure
      serviceConfig.Restart = "always";

      # Ensure the service starts after the network is up
      after = [ "network.target" ];
    };
  };
}
