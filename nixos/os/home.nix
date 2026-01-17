{ config, pkgs, lib, ... }: {
  home.stateVersion = "24.11";

  home.packages = with pkgs; [ pkgs.qitechPackages.electron ];

  # Add autostart entry for the QiTech electron app
  xdg.configFile."autostart/de.qitech.control-electron.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=QiTech Control
    Comment=QiTech Control
    Exec=qitech-control-electron
    Icon=de.qitech.control-electron
    Terminal=false
    StartupWMClass=QiTech Control
    X-GNOME-Autostart-enabled=true
    X-GNOME-Autostart-Phase=Applications
  '';

  dconf.settings = {
    # Set GNOME wallpaper
    "org/gnome/desktop/background" = {
      picture-uri =
        "https://i.postimg.cc/Z5XJtNMW/qitech-industries-wallpaper.jpg";
      picture-uri-dark =
        "https://i.postimg.cc/Z5XJtNMW/qitech-industries-wallpaper.jpg";
      picture-options = "zoom";
    };

    # Enable on-screen keyboard 
    "org/gnome/desktop/a11y/applications" = { screen-keyboard-enabled = true; };

    # Configure on-screen keyboard (optional)
    "org/gnome/desktop/a11y" = { always-show-universal-access-status = true; };

    # Disable screen blanking/timeout
    "org/gnome/desktop/session" = {
      idle-delay =
        lib.gvariant.mkUint32 0; # Use uint32 format to ensure proper type
    };

    # Disable automatic suspend and screen dimming
    "org/gnome/settings-daemon/plugins/power" = {
      sleep-inactive-ac-type = "nothing";
      sleep-inactive-battery-type = "nothing";
      power-button-action = "nothing";
      idle-dim = false;
      sleep-inactive-ac-timeout = lib.gvariant.mkUint32 0;
      sleep-inactive-battery-timeout = lib.gvariant.mkUint32 0;
      ambient-enabled = false;
    };

    # Disable screen lock
    "org/gnome/desktop/screensaver" = {
      lock-enabled = false;
      idle-activation-enabled = false;
      lock-delay = lib.gvariant.mkUint32 0;
    };

    # Interface settings with hot-corners disabled
    "org/gnome/desktop/interface" = {
      enable-animations = true;
      show-battery-percentage = true;
      enable-hot-corners =
        false; # Prevent activities from opening with hot corners
    };

    # Show dock on all monitors and always visible
    "org/gnome/shell/extensions/dash-to-dock" = {
      dock-position = "BOTTOM";
      extend-height = false;
      dock-fixed = true;
      autohide = false;
      intellihide = false;
      multi-monitor = true; # Show dock on all monitors
      isolate-monitors = false; # Don't isolate workspaces per monitor
      isolate-workspaces = false; # Don't isolate dock content per workspace
    };

    # Configure display settings
    "org/gnome/mutter" = {
      workspaces-only-on-primary = false; # Show workspaces on all monitors
      dynamic-workspaces = false; # Disable dynamic workspaces
    };

    "org/gnome/desktop/wm/preferences" = {
      num-workspaces = 1; # Set a fixed number of workspaces (default: 4)
    };

    "org/gnome/desktop/lockdown" = {
      disable-lock-screen = true; # Disable lock screen
      disable-log-out =
        false; # Enable logout option (To be able to reboot manually)
      disable-user-switching = true; # Disable user switching
      disable-screensaver = true; # Disable screensaver
      user-adminstration-disabled = true; # Disable user administration
    };

    "org/gnome/shell" = {
      always-show-log-out = true;
      enabled-extensions =
        [ "dash-to-dock@micxgx.gmail.com" "no-overview@fthx" ];
      favorite-apps = [
        "de.qitech.control-electron.desktop" # The desktop entry from the QiTech app
        "org.gnome.Settings.desktop"
      ];
      disable-user-extensions = false;
      disable-extension-version-validation = true;
      welcome-dialog-last-shown-version =
        "999.999.999"; # Prevents welcome screen
      looking-glass-history = [ ];
    };

    # Disable automatic Activities overview on startup
    "org/gnome/shell/extensions/ding" = {
      start-corner =
        "top-left"; # This can help with preventing Activities from opening
    };
  };
}
