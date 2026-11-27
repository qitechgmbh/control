{
  config,
  lib,
  pkgs,
  ...
}:

# FIXME: bad practice
with lib;

let
  cfg = config.services.qitech;
in
{
  options.services.qitech = {
    enable = mkEnableOption "QiTech Control";

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to open ports in the firewall for the QiTech server";
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
    assertions = [
      {
        assertion = cfg.package != null;
        message = "No QiTech server package available. Please add it to your pkgs or explicitly set services.qitech.package.";
      }
    ];

    networking.firewall = mkIf cfg.openFirewall { allowedTCPPorts = [ 443 ]; };

    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "QiTech service user";
      extraGroups = [
        "realtime"
        "plugdev"
        "dialout"
        "uucp"
      ];
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

    # Realtime slice: the only cgroup allowed on the isolated CPU 3 (system.slice and user.slice are
    # limited to 0-2 in configuration.nix). The server starts with the manager's default affinity
    # (0-2) and only its RT threads pin themselves to CPU 3.
    systemd.slices.rt = {
      description = "Realtime Slice";
      sliceConfig.AllowedCPUs = "0-3";
    };

    # Configure the systemd service
    systemd.services.qitech-control-server = {
      description = "QiTech Control Server";
      wantedBy = [ "multi-user.target" ];
      # Start only once ClickHouse is ready (its unit is Type=notify): `init_db` needs it at
      # startup. `wants`, not `requires`, so a ClickHouse restart doesn't stop the machines.
      after = [
        "network.target"
        "clickhouse.service"
      ];
      wants = [ "clickhouse.service" ];

      serviceConfig = {
        Type = "simple";
        Slice = "rt.slice";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/qitech_control";
        Restart = "always";
        RestartSec = "10s";

        # Capabilities
        CapabilityBoundingSet = "CAP_NET_RAW CAP_IPC_LOCK CAP_NET_ADMIN CAP_SYS_NICE CAP_DAC_OVERRIDE";
        AmbientCapabilities = "CAP_NET_RAW CAP_IPC_LOCK CAP_NET_ADMIN CAP_SYS_NICE CAP_DAC_OVERRIDE";

        # Hardening options
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        StateDirectory = "qitech";

        # Open only /proc/irq explicitly
        ReadWritePaths = [ "/proc/irq" ];
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = false;

        # Must disable this to allow /proc/irq writes
        ProtectKernelTunables = false;
        ProtectControlGroups = true;
        RestrictAddressFamilies = "AF_UNIX AF_INET AF_INET6 AF_NETLINK AF_PACKET";
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
    # needed so dnsmasq can get managed by server
    security.polkit.extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (action.id == "org.freedesktop.systemd1.manage-units" &&
            action.lookup("unit") == "dnsmasq.service" &&
            subject.user == "${cfg.user}") {
          return polkit.Result.YES;
        }
      });
    '';
  };
}
