{
  inputs,
  lib,
  pkgs,
  ...
}:
let
  gitInfo = import ../gitInfo.nix { inherit pkgs; };
in
{
  boot.kernelPackages = pkgs.linuxPackages_6_18;
  # Disable filesystems with Out-of-tree kernel modules
  boot.supportedFilesystems.zfs = lib.mkForce false;
  boot.supportedFilesystems.bcachefs = lib.mkForce false;
  boot.kernelParams = [
    # Realtime Preemption
    "preempt=full"

    # Graphical
    "logo.nologo" # Remove kernel logo during boot

    # Performance

    # Specific Vulnerabilities Addressed by Mitigations:
    # - Spectre variants (V1, V2, V4, SWAPGS, SpectreRSB, etc.)
    # - Meltdown (Rogue Data Cache Load)
    # - Foreshadow/L1TF (L1 Terminal Fault)
    # - Microarchitectural Data Sampling (MDS, RIDL, Fallout, ZombieLoad)
    # - SRBDS (Special Register Buffer Data Sampling)
    # - TSX Asynchronous Abort (TAA)
    # - iTLB Multihit
    # - And others as they're discovered and mitigated
    #
    # With mitigations=off
    # - PROS: Maximum performance, equivalent to pre-2018 behavior
    # - CONS: Vulnerable to Spectre, Meltdown, Foreshadow, ZombieLoad, etc.
    #         Should ONLY be used in completely trusted environments
    # - Improves performance by 7-43%
    "mitigation=off"
    "intel_pstate=performance" # Intel CPU-specific performance mode (if applicable)

    # Memory Management
    "transparent_hugepage=always" # Use larger memory pages for memory intense applications
    "nmi_watchdog=0" # Disable NMI watchdog for reduced CPU overhead and realtime execution

    # High-throughput ethernet parameters
    "pcie_aspm=off" # Disable PCIe power management for NICs
    "intel_iommu=off" # Disable IOMMU (performance gain)

    # Reliability
    "panic=10" # Auto-reboot 10 seconds after kernel panic
    "oops=panic" # Treat kernel oops as panic for auto-recovery
    "usbcore.autosuspend=-1" # Possibly fixes dre disconnect issue?

    "isolcpus=2,3" # Isolate cpus 2 and 3 from scheduler for better latency, 2 runs ethercatthread and 3 runs server control-loop
    "nohz_full=2,3" # In this mode, the periodic scheduler tick is stopped when only one task is running, reducing kernel interruptions on those CPUs.
    "rcu_nocbs=2,3" # Moves RCU (Read-Copy Update) callback processing away from CPUs 2 and 3.

  ];

  boot.kernel.sysctl = {
    "kernel.panic_on_oops" = 1; # Reboot on kernel oops
    "kernel.panic" = 10; # Reboot after 10 seconds on panic
    "vm.swappiness" = 10; # Reduce swap usage
    "kernel.sysrq" = 1; # Enable SysRq for emergency control
  };

  nix = {
    package = pkgs.nixVersions.stable;
    settings = {
      experimental-features = "nix-command flakes";
    };
  };

  # Create a realtime group
  users.groups.realtime = { };

  # Configure real-time privileges
  security.pam.loginLimits = [
    {
      domain = "@realtime";
      type = "-";
      item = "rtprio";
      value = "99";
    }
    {
      domain = "@realtime";
      type = "-";
      item = "memlock";
      value = "unlimited";
    }
    {
      domain = "@realtime";
      type = "-";
      item = "nice";
      value = "-20";
    }
  ];

  networking.hostName = "nixos";

  # Enable networking
  networking.networkmanager.enable = true;
  networking.wireless.enable = lib.mkImageMediaOverride false;

  # Enable the X11 windowing system.
  services.displayManager.gdm = {
    enable = true;
    autoSuspend = false;
    wayland = true;
  };
  services.desktopManager.gnome.enable = true;

  services.caddy = {
    enable = true;
    # This puts the import at the TOP of the Caddyfile (Global Scope)
    extraConfig = ''
      import /var/lib/caddy/auth_snippet.conf
    '';

    virtualHosts.":443" = {
      extraConfig = ''
        import machine_basic_auth

        reverse_proxy localhost:3001

        tls internal {
          on_demand
        }

      '';
    };
  };

  systemd.services.caddy.serviceConfig.ReadOnlyPaths = [ "/var/lib/caddy/auth_snippet.conf" ];
  #services.caddy = {
  #  enable = true;
  #  virtualHosts."localhost".extraConfig = ''
  #    respond "Hello, world!"
  #  '';
  #};

  # Disable sleep/suspend
  systemd.targets.sleep.enable = false;
  systemd.targets.suspend.enable = false;
  systemd.targets.hibernate.enable = false;
  systemd.targets.hybrid-sleep.enable = false;

  # Additional power management settings
  powerManagement = {
    enable = true;
    cpuFreqGovernor = "performance";
    # Disable power throttling for peripheral devices
    powertop.enable = false;
  };

  services.logind = {
    # Structured settings for logind.conf
    settings = {
      Login = {
        HandlePowerKey = "ignore";
        HandleSuspendKey = "ignore";
        HandleHibernateKey = "ignore";
        HandleLidSwitch = "ignore";
        IdleAction = "ignore";
      };
    };
  };

  # Enable sound with pipewire.
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  # Enable graphics acceleration
  hardware.graphics.enable = true;

  services.libinput.enable = true;
  services.libinput.touchpad.tapping = true;
  services.touchegg.enable = true;

  # Enable the QiTech Control server
  services.qitech = {
    enable = true;
    user = "qitech-service";
    group = "qitech-service";
    package = pkgs.qitechPackages.server;
  };

  users.users.qitech = {
    isNormalUser = true;
    description = "QiTech HMI";
    extraGroups = [
      "networkmanager"
      "wheel"
      "realtime"
      "wireshark"
    ];
  };

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.users.qitech = import ./home.nix;
  home-manager.extraSpecialArgs = {
    inherit inputs;
  };

  security.sudo.wheelNeedsPassword = false;
  # The equivalent for pkexec (graphical sudo)
  security.polkit.extraConfig = ''
    polkit.addRule(function(action, subject) {
      if (subject.isInGroup("wheel")) {
        return polkit.Result.YES;
      }
    });
  '';

  # Enable automatic login for the user.
  services.displayManager.autoLogin.enable = true;
  services.displayManager.autoLogin.user = "qitech";

  # Workaround for GNOME autologin: https://github.com/NixOS/nixpkgs/issues/103746#issuecomment-945091229
  systemd.services."getty@tty1".enable = false;
  systemd.services."autovt@tty1".enable = false;

  # Install firefox.
  programs.firefox.enable = true;

  # Enable Wireshark with proper permissions
  programs.wireshark.enable = true;

  documentation.doc.enable = false;
  services.gnome.core-apps.enable = false;
  environment.gnome.excludePackages = [ pkgs.gnome-tour ];
  environment.systemPackages = with pkgs; [
    # Bare minimum gnome desktop
    gnome-console
    gnome-tweaks
    loupe
    mpv
    nautilus
    gnomeExtensions.no-overview
    # Utilities
    git
    btop
    htop
    wireshark
    pciutils
    neofetch
    caddy
    # QiTech Frontend
    pkgs.qitechPackages.electron
  ];

  # Set system wide env variables
  environment.variables = {
    QITECH_OS = "true";
    QITECH_OS_GIT_TIMESTAMP = gitInfo.gitTimestamp;
    QITECH_OS_GIT_COMMIT = gitInfo.gitCommit;
    QITECH_OS_GIT_ABBREVIATION = gitInfo.gitAbbreviation;
    QITECH_OS_GIT_URL = gitInfo.gitUrl;
  };

  # Set revision label
  system.nixos.label = "${gitInfo.gitAbbreviationEscaped}_${gitInfo.gitCommit}";

  networking.firewall.allowedUDPPorts = [
    53
    67
    69
  ];
  networking.firewall.allowedTCPPorts = [ 443 ];

  i18n.supportedLocales = [ "all" ];

  # Dont edit
  system.stateVersion = "24.11";
}
