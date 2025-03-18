{ config, pkgs, ... }: {
  home.stateVersion = "24.11";

  # Add autostart entry for the QiTech electron app
  xdg.configFile."autostart/de.qitech.control-electron.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=QiTech Control
    Comment=QiTech Industries Control Software
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
      picture-uri = "https://i.postimg.cc/0N61BwSp/qitech-industries-wallpaper.jpg";
      picture-uri-dark = "https://i.postimg.cc/0N61BwSp/qitech-industries-wallpaper.jpg";
      picture-options = "zoom";
    };

    # Enable on-screen keyboard 
    "org/gnome/desktop/a11y/applications" = {
      screen-keyboard-enabled = true;
    };
    
    # Configure on-screen keyboard (optional)
    "org/gnome/desktop/a11y" = {
      always-show-universal-access-status = true;
    };

    # Disable screen blanking/timeout
    "org/gnome/desktop/session" = {
      idle-delay = 0;  # 0 seconds = never
    };
    
    # Disable automatic suspend
    "org/gnome/settings-daemon/plugins/power" = {
      sleep-inactive-ac-type = "nothing";
      sleep-inactive-battery-type = "nothing";
      power-button-action = "nothing";
    };
    
    # Disable screen lock
    "org/gnome/desktop/screensaver" = {
      lock-enabled = false;
      idle-activation-enabled = false;
    };
    
    # Disable screen dimming
    "org/gnome/settings-daemon/plugins/power" = {
      idle-dim = false;
    };

    # Show dock on all monitors and always visible
    "org/gnome/shell/extensions/dash-to-dock" = {
      dock-position = "BOTTOM";
      extend-height = false;
      dock-fixed = true;
      autohide = false;
      intellihide = false;
      multi-monitor = true;  # Show dock on all monitors
      isolate-monitors = false;  # Don't isolate workspaces per monitor
      isolate-workspaces = false;  # Don't isolate dock content per workspace
    };
    
    # Configure display settings
    "org/gnome/mutter" = {
      workspaces-only-on-primary = false;  # Show workspaces on all monitors
      dynamic-workspaces = false;  # Disable dynamic workspaces
    };
    
    "org/gnome/desktop/wm/preferences" = {
      num-workspaces = 1;  # Set a fixed number of workspaces (default: 4)
    };
    
    "org/gnome/shell" = {
      always-show-log-out = true;
      enabled-extensions = [
        "dash-to-dock@micxgx.gmail.com"
      ];
      favorite-apps = [
        "de.qitech.control-electron.desktop"  # The desktop entry from the QiTech app
        "org.gnome.Settings.desktop"
        "org.gnome.Terminal.desktop"
      ];
    };
  };
}
