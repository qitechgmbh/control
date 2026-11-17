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

    igh = {
      enable = mkEnableOption "the IgH EtherCAT master (kernel modules, ethercat CLI and /dev/EtherCAT* access)";

      device = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "enp2s0";
        description = ''
          Default network interface (name or MAC address) for EtherCAT master 0.
          Overridden at runtime by the contents of /var/lib/ethercat/device, so the
          interface can be changed on a panel without a rebuild:
            echo enp3s0 | sudo tee /var/lib/ethercat/device
            sudo systemctl restart ethercat
        '';
      };
    };
  };

  config = mkIf cfg.enable (mkMerge [
    (mkIf cfg.igh.enable {
      # IgH EtherCAT master: ec_master + generic NIC driver (the native NIC
      # drivers are only built by nixpkgs for older kernel versions).
      boot.extraModulePackages = [
        (config.boot.kernelPackages.callPackage pkgs.ethercat.kernelModule { })
      ];

      environment.systemPackages = [ pkgs.ethercat ];

      # Not loaded via boot.kernelModules: ec_generic only scans the interfaces that
      # exist at load time and ec_master binds by MAC, so load both once the chosen
      # interface is present and up.
      systemd.services.ethercat = {
        description = "IgH EtherCAT master";
        wantedBy = [ "multi-user.target" ];
        before = [ "qitech-control-server.service" ];
        path = [
          pkgs.kmod
          pkgs.iproute2
          pkgs.coreutils
          pkgs.gnugrep
        ];
        serviceConfig = {
          Type = "oneshot";
          RemainAfterExit = true;
          StateDirectory = "ethercat";
          TimeoutStartSec = "60s";
        };
        script = ''
          DEVICE=""
          if [ -s /var/lib/ethercat/device ]; then
            DEVICE=$(tr -d '[:space:]' < /var/lib/ethercat/device)
          fi
          if [ -z "$DEVICE" ]; then
            DEVICE=${escapeShellArg (if cfg.igh.device == null then "" else cfg.igh.device)}
          fi
          if [ -z "$DEVICE" ]; then
            echo "No EtherCAT interface configured. Set one with:"
            echo "  echo <interface-or-mac> | sudo tee /var/lib/ethercat/device && sudo systemctl restart ethercat"
            exit 1
          fi

          # Resolve interface name <-> MAC, waiting for the NIC driver to create it.
          IFACE=""
          for _ in $(seq 1 30); do
            if echo "$DEVICE" | grep -qiE '^([0-9a-f]{2}:){5}[0-9a-f]{2}$'; then
              for d in /sys/class/net/*; do
                if [ "$(cat "$d/address")" = "$(echo "$DEVICE" | tr 'A-F' 'a-f')" ]; then
                  IFACE=$(basename "$d")
                fi
              done
            elif [ -e "/sys/class/net/$DEVICE" ]; then
              IFACE="$DEVICE"
            fi
            [ -n "$IFACE" ] && break
            sleep 1
          done
          if [ -z "$IFACE" ]; then
            echo "EtherCAT interface '$DEVICE' not found. Available interfaces:"
            ls /sys/class/net
            exit 1
          fi
          MAC=$(cat "/sys/class/net/$IFACE/address")

          echo "Using $IFACE ($MAC) as EtherCAT main device"
          ip link set dev "$IFACE" up
          modprobe ec_master main_devices="$MAC"
          modprobe ec_generic
        '';
        preStop = ''
          modprobe -r ec_generic || true
          modprobe -r ec_master
        '';
      };

      services.udev.extraRules = ''
        KERNEL=="EtherCAT[0-9]*", MODE="0660", GROUP="${cfg.group}"
      '';
    })
    {
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

      # Configure the systemd service
      systemd.services.qitech-control-server = {
        description = "QiTech Control Server";
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];

        serviceConfig = {
          Type = "simple";
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
    }
  ]);
}
