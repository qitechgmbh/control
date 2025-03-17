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
      default = "qitech";
      description = "User account under which the service runs";
    };
    
    group = mkOption {
      type = types.str;
      default = "qitech";
      description = "Group under which the service runs";
    };
    
    port = mkOption {
      type = types.port;
      default = 8000;  # Adjust to the actual default port
      description = "Port on which the QiTech server listens";
    };
  };

  config = mkIf cfg.enable {
    # Create user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "QiTech service user";
    };
    
    users.groups.${cfg.group} = {};
    
    # Install the packages
    environment.systemPackages = [
      qitechPackages.electron
    ];
    
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
        
        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = false;  # Likely needs access to EtherCAT devices
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = "AF_UNIX AF_INET AF_INET6 AF_NETLINK";
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = false;  # May need JIT compilation
        CapabilityBoundingSet = "CAP_NET_RAW";  # Needed for raw socket access for EtherCAT
      };
      
      environment = {
        PORT = toString cfg.port;
      };
    };
    
    # Open firewall if requested
    networking.firewall = mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.port ];
    };
    
    # Desktop integration
    xdg.mime.enable = true;
    xdg.icons.enable = true;
  };
}
