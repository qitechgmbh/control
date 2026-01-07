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

    # Configure the systemd service
    systemd.services.qitech-control-server = {
      description = "QiTech Control Server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/server";
        Restart = "always";
        RestartSec = "10s";

        # Capabilities
        CapabilityBoundingSet =
          "CAP_NET_RAW CAP_IPC_LOCK CAP_NET_ADMIN CAP_SYS_NICE CAP_DAC_OVERRIDE";
        AmbientCapabilities =
          "CAP_NET_RAW CAP_IPC_LOCK CAP_NET_ADMIN CAP_SYS_NICE CAP_DAC_OVERRIDE";

        # Hardening options
        NoNewPrivileges = true;
        ProtectSystem = "strict";

        # Open only /proc/irq explicitly
        ReadWritePaths = [ "/proc/irq" ];
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = false;

        # Must disable this to allow /proc/irq writes
        ProtectKernelTunables = false;
        ProtectControlGroups = true;
        RestrictAddressFamilies =
          "AF_UNIX AF_INET AF_INET6 AF_NETLINK AF_PACKET";
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = false;

        # Logging
        StandardOutput = "journal";
        StandardError = "journal";
        SyslogIdentifier = "qitech-control-server";
      };

      environment = {
        RUST_BACKTRACE = "full";
        RUST_LOG = "info";
      };
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
