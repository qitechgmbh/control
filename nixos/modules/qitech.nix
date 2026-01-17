{ config, lib, pkgs, ... }:

with lib;

let cfg = config.services.qitech;
in {
  options.services.qitech = {
    enable = mkEnableOption "QiTech Control";

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description =
        "Whether to open ports in the firewall for the QiTech server";
    };

    user = mkOption {
      type = types.str;
      default = "qitech-service";
      description = "User account under which the service runs";
    };

    group = mkOption {
      type = types.str;
      default = "qitech-service";
      description = "Group under which the service runs";
    };

    package = mkOption {
      type = types.package;
      default = pkgs.qitech-control-server or null;
      description = "The QiTech server package to use";
    };
  };

  config = mkIf cfg.enable {
    assertions = [{
      assertion = cfg.package != null;
      message =
        "No QiTech server package available. Please add it to your pkgs or explicitly set services.qitech.package.";
    }];

    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "QiTech service user";
      extraGroups = [ "realtime" "plugdev" "dialout" "uucp" ];
    };

    users.groups.${cfg.group} = { };

    # Install the Electron app system-wide
    environment.systemPackages = [ pkgs.qitechPackages.electron ];

    # Configure udev rules for EtherCAT device access
    services.udev.extraRules = ''
      # Allow access to network devices for ethercrab
      SUBSYSTEM=="net", ACTION=="add", ATTR{address}=="*", TAG+="uaccess", TAG+="udev-acl", GROUP="${cfg.group}"

      # USB device access if needed
      SUBSYSTEM=="usb", ATTRS{idVendor}=="*", ATTRS{idProduct}=="*", MODE="0660", GROUP="${cfg.group}"
    '';

    # Generate systemd service file from template and install it
    systemd.packages = [
      (pkgs.writeTextFile {
        name = "qitech-control-server-service";
        destination = "/etc/systemd/system/qitech-control-server.service";
        text = builtins.readFile (pkgs.substituteAll {
          src = ./../services/qitech-control-server.service;
          user = cfg.user;
          group = cfg.group;
          execstart = "${cfg.package}/bin/server";
        });
      })
    ];

    # Enable the service
    systemd.services.qitech-control-server = {
      wantedBy = [ "multi-user.target" ];
      enable = true;
    };

    # Add real-time privileges
    security.pam.loginLimits = [
      {
        domain = cfg.user;
        type = "-";
        item = "rtprio";
        value = "99";
      }
      {
        domain = cfg.user;
        type = "-";
        item = "memlock";
        value = "unlimited";
      }
      {
        domain = cfg.user;
        type = "-";
        item = "nice";
        value = "-20";
      }
    ];

    # Open firewall if requested
    networking.firewall = mkIf cfg.openFirewall { allowedTCPPorts = [ 3001 ]; };

    # Desktop integration
    xdg.mime.enable = true;
    xdg.icons.enable = true;
  };
}
