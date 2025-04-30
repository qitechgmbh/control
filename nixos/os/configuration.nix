# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{ config, pkgs, installInfo, ... }:

{
  imports =
    [ # Include the results of the hardware scan.
      /etc/nixos/hardware-configuration.nix
    ];

  # Bootloader.
  boot.loader.systemd-boot = {
    enable = true;
    consoleMode = "max";  # Use the highest available resolution
  };
  boot.loader.efi.canTouchEfiVariables = true;

  boot.kernelPackages = pkgs.linuxPackages_latest;

  nix = {
    package = pkgs.nixVersions.stable;
    extraOptions = ''
      experimental-features = nix-command flakes
    '';
    settings = {
      sandbox = false;
    };
  };

  # Create a realtime group
  users.groups.realtime = {};

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

  networking.hostName = "nixos"; # Define your hostname.
  # networking.wireless.enable = true; # Enables wireless support via wpa_supplicant.

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

  # Disable sleep/suspend
  systemd.targets.sleep.enable = false;
  systemd.targets.suspend.enable = false;
  systemd.targets.hibernate.enable = false;
  systemd.targets.hybrid-sleep.enable = false;

  # Additional power management settings
  powerManagement.enable = false;

  # Ensure all power management is disabled
  services.logind = {
    lidSwitch = "ignore";
    extraConfig = ''
      HandlePowerKey=ignore
      HandleSuspendKey=ignore
      HandleHibernateKey=ignore
      HandleLidSwitch=ignore
      IdleAction=ignore
    '';
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
    # If you want to use JACK applications, uncomment this
    #jack.enable = true;

    # use the example session manager (no others are packaged yet so this is enabled by default,
    # no need to redefine it in your config for now)
    #media-session.enable = true;
  };

  # Enable graphics acceleration
  hardware.graphics.enable = true;

  # Enable touchpad support (enabled default in most desktopManager).
  services.libinput.enable = true;
  services.libinput.touchpad.tapping = true;
  services.touchegg.enable = true;

  # Enable the QiTech Control server
  services.qitech = {
    enable = true;
    openFirewall = true;
    user = "qitech-service";
    group = "qitech-service"; 
    port = 3001;
    package = pkgs.qitechPackages.server;
  };

  # Enable logging with journald for the QiTech Control server service
  systemd.services.qitech = {
    serviceConfig = {
      StandardOutput = "journal";
      StandardError = "journal";
      Restart = "always";
      SyslogIdentifier = "qitech-control-server";
    };
    environment = {
      RUST_BACKTRACE = "1";
      RUST_LOG = "debug";
    };
  };

  users.users.qitech = {
    isNormalUser = true;
    description = "QiTech Industries";
    extraGroups = [ "networkmanager" "wheel" "realtime" ];
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

  # List packages installed in system profile. To search, run:
  # $ nix search wget
  environment.systemPackages = with pkgs; [
  #  vim # Do not forget to add an editor to edit configuration.nix! The Nano editor is also installed by default.
  #  wget
    gnome-tweaks
    gnome-extension-manager
    gnomeExtensions.dash-to-dock
    git
    pkgs.qitechPackages.electron
    htop
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
    gedit # text editor
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
    QITECH_OS_GIT_TIMESTAMP = installInfo.gitTimestamp;
    QITECH_OS_GIT_COMMIT = installInfo.gitCommit;
    QITECH_OS_GIT_ABBREVIATION = installInfo.gitAbbreviation;
    QITECH_OS_GIT_URL = installInfo.gitUrl;
  };

  # Set revision labe;
  system.nixos.label = "${installInfo.gitAbbreviationEscaped}_${installInfo.gitCommit}";
  
  # Some programs need SUID wrappers, can be configured further or are
  # started in user sessions.
  # programs.mtr.enable = true;
  # programs.gnupg.agent = {
  #   enable = true;
  #   enableSSHSupport = true;
  # };

  # List services that you want to enable:

  # Enable the OpenSSH daemon.
  # services.openssh.enable = true;

  # Open ports in the firewall.
  # networking.firewall.allowedTCPPorts = [ ... ];
  # networking.firewall.allowedUDPPorts = [ ... ];
  # Or disable the firewall altogether.
  # networking.firewall.enable = false;

  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
  system.stateVersion = "24.11"; # Did you read the comment?

}
