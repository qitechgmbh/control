{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.qitech;
  qitechPackages = (import ../. { inherit pkgs; }).packages;
in {
  options.services.qitech = {
    enable = mkEnableOption "QiTech Industries Control Software";
    
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
    
    port = mkOption {
      type = types.port;
      default = 8000;
      description = "Port on which the QiTech server listens";
    };
  };

  config = mkIf cfg.enable {
    # Always create a dedicated system user
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "QiTech service user";
      # Add to groups needed for hardware access
      extraGroups = [ "realtime" "plugdev" "dialout" "uucp" ];
    };
    
    users.groups.${cfg.group} = {};
    
    # Install the packages
    environment.systemPackages = [
      qitechPackages.electron
    ];
    
    # Configure udev rules for EtherCAT device access
    services.udev.extraRules = ''
      # Allow access to network devices for ethercrab
      SUBSYSTEM=="net", ACTION=="add", ATTR{address}=="*", TAG+="uaccess", TAG+="udev-acl", GROUP="${cfg.group}"
      
      # USB device access if needed
      SUBSYSTEM=="usb", ATTRS{idVendor}=="*", ATTRS{idProduct}=="*", MODE="0660", GROUP="${cfg.group}"
    '';
    
    # Configure the systemd service
    systemd.services.qitech-server = {
      description = "QiTech Industries Control Software Server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      
      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${qitechPackages.server}/bin/server";
        Restart = "on-failure";
        
        # Grant specific capabilities needed for EtherCAT
        CapabilityBoundingSet = "CAP_NET_RAW CAP_NET_ADMIN CAP_SYS_NICE";
        AmbientCapabilities = "CAP_NET_RAW CAP_NET_ADMIN CAP_SYS_NICE";
        
        # Hardening that's still compatible with hardware access
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = false;  # Need access to devices
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = "AF_UNIX AF_INET AF_INET6 AF_NETLINK AF_PACKET";
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = false;  # May need JIT compilation
      };
      
      environment = {
        PORT = toString cfg.port;
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
    networking.firewall = mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.port ];
    };
    
    # Desktop integration
    xdg.mime.enable = true;
    xdg.icons.enable = true;
  };
}
