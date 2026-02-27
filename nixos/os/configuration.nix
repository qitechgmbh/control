{ config, pkgs, ... }:

let gitInfo = import ../gitInfo.nix { inherit pkgs; };
in {
  imports = [
    (if builtins.pathExists "/etc/nixos/hardware-configuration.nix" then
      /etc/nixos/hardware-configuration.nix
    else
      ./ci-hardware-configuration.nix)
  ];

  # Bootloader.
  boot.loader.systemd-boot = {
    enable = true;
    consoleMode = "max"; # Use the highest available resolution
  };
  boot.loader.efi.canTouchEfiVariables = true;
  boot.kernelPackages = pkgs.linuxPackages_6_18;
  boot.kernelModules = [ "i915" ];

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

    "isolcpus=3" # Isolate cpu 3 from scheduler for better latency, 3 runs server control-loop
    "nohz_full=3" # In this mode, the periodic scheduler tick is stopped when only one task is running, reducing kernel interruptions on those CPUs.
    "rcu_nocbs=3" # Moves RCU (Read-Copy Update) callback processing away from CPU 3.

  ];

  boot.kernel.sysctl = {
    "kernel.panic_on_oops" = 1; # Reboot on kernel oops
    "kernel.panic" = 10; # Reboot after 10 seconds on panic
    "vm.swappiness" = 10; # Reduce swap usage
    "kernel.sysrq" = 1; # Enable SysRq for emergency control
  };

  nix = {
    package = pkgs.nixVersions.stable;
    extraOptions = ''
      experimental-features = nix-command flakes
    '';
    settings = { sandbox = false; };
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

  # Configure network proxy if necessary
  # networking.proxy.default = "http://user:password@proxy:port/";
  # networking.proxy.noProxy = "127.0.0.1,localhost,internal.domain";

  # Enable networking
  networking.networkmanager.enable = true;

  # Set your time zone.
  time.timeZone = "UTC";

  # Select internationalisation properties.
  # we use en_DK for english texts but metric units and 24h time
  i18n.defaultLocale = "en_DK.UTF-8";

  i18n.extraLocaleSettings = {
    LC_ADDRESS = "en_DK.UTF-8";
    LC_IDENTIFICATION = "en_DK.UTF-8";
    LC_MEASUREMENT = "en_DK.UTF-8";
    LC_MONETARY = "en_DK.UTF-8";
    LC_NAME = "en_DK.UTF-8";
    LC_NUMERIC = "en_DK.UTF-8";
    LC_PAPER = "en_DK.UTF-8";
    LC_TELEPHONE = "en_DK.UTF-8";
    LC_TIME = "en_DK.UTF-8";
  };

  # Enable the X11 windowing system.
  services.xserver.enable = true;
  services.xserver.displayManager.gdm = {
    enable = true;
    autoSuspend = false;
    wayland = true;
  };
  services.xserver.desktopManager.gnome.enable = true;

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
    # This remains for backward compatibility/high-level override
    lidSwitch = "ignore";

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



  # Configure keymap in X11
  services.xserver.xkb = {
    layout = "de";
    variant = "";
  };

  # Configure console keymap
  console.keyMap = "de";

  # Enable CUPS to print documents.
  services.printing.enable = false;

  # Enable sound with pipewire.
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  # Enable graphics acceleration
  hardware.graphics = {
    enable = true;
    extraPackages = with pkgs; [ mesa ];
  };

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
    extraGroups = [ "networkmanager" "wheel" "realtime" "wireshark" ];
    packages = with pkgs; [ ];
  };

  security.sudo.wheelNeedsPassword = false;

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
  programs.wireshark.package = pkgs.wireshark;

  # List packages installed in system profile. To search, run:
  # $ nix search wget
  environment.systemPackages = with pkgs; [
    #  vim # Do not forget to add an editor to edit configuration.nix! The Nano editor is also installed by default.
    #  wget
    gnome-tweaks
    gnome-extension-manager
    gnomeExtensions.dash-to-dock
    # Extension to disable activities overview on login
    gnomeExtensions.no-overview
    git
    pkgs.qitechPackages.electron
    htop
    wireshark
    pciutils
    neofetch
    caddy
  ];

  xdg.portal.enable = true;
  xdg.portal.extraPortals = [ pkgs.xdg-desktop-portal-gtk ];

  environment.gnome.excludePackages = (with pkgs; [
    atomix # puzzle game
    baobab # disk usage analyzer
    cheese # webcam tool
    eog # image viewer
    epiphany # web browser
    evince # document viewer
    geary # email reader
    simple-scan # document scanner
    gnome-characters
    gnome-music
    gnome-photos
    gnome-terminal
    gnome-tour
    gnome-calculator
    gnome-calendar
    gnome-contacts
    gnome-maps
    gnome-weather
    hitori # sudoku game
    iagno # go game
    tali # poker game
    totem # video player
    seahorse # password manager
  ]);

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
  
  networking.firewall.allowedUDPPorts = [ 53 67 69 ];
  networking.firewall.allowedTCPPorts = [ 443 ];

  system.stateVersion = "24.11"; # Did you read the comment?

}
