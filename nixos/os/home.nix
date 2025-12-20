{ config, pkgs, lib, ... }: {
  home.stateVersion = "25.11";

  home.packages = with pkgs; [
    pkgs.qitechPackages.electron
  ];

  # Add autostart entry for the QiTech electron app
  # xdg.configFile."autostart/de.qitech.control-electron.desktop".text = ''
  #   [Desktop Entry]
  #   Type=Application
  #   Name=QiTech Control
  #   Comment=QiTech Control
  #   Exec=qitech-control-electron
  #   Icon=de.qitech.control-electron
  #   Terminal=false
  #   StartupWMClass=QiTech Control
  #   X-GNOME-Autostart-enabled=true
  #   X-GNOME-Autostart-Phase=Applications
  # '';

  # XFCE desktop environment settings
  xfconf.settings = {
    "xfce4-desktop" = {
      # Set wallpaper for the primary display
      "backdrop/screen0/monitor0/image-path" = "https://i.postimg.cc/Z5XJtNMW/qitech-industries-wallpaper.jpg";
      "backdrop/screen0/monitor0/image-show" = true;
      "backdrop/screen0/monitor0/image-style" = 5; # 5 = Zoomed (fit to screen)
    };

    "xfce4-power-manager" = {
      # Disable automatic suspend, dimming, and screen blank
      "inactivity-on-ac" = 14; # 14 = Do nothing
      "inactivity-on-battery" = 14;
      "blank-on-ac" = 0;
      "blank-on-battery" = 0;
      "dpms-enabled" = false;
      "brightness-on-ac" = 100;
      "brightness-on-battery" = 100;
    };

    "xfce4-session" = {
      # Disable session auto lock
      "lock-screen-suspend-hibernate" = false;
      "logout-prompt" = true;
    };

    "xfce4-screensaver" = {
      "lock-enabled" = false;
      "mode" = 0; # 0 = Blank, 1 = Random, 2 = Single image
    };

    "xfce4-panel" = {
      # Always show panel, don’t autohide
      "autohide-behavior" = 0; # 0 = Never autohide
      "show-window-list" = true;
      "show-desktop-button" = true;
    };

    "xfwm4" = {
      # Window manager preferences
      "workspace_count" = 1;
      "wrap_workspaces" = false;
      "wrap_layout" = false;
      "cycle_workspaces" = false;
      "workspace_names" = [ "Main" ];
    };

    "xfce4-keyboard-shortcuts" = {
      # Disable hot corners or any special shortcuts
      "/commands/custom/<Super>p" = ""; # Example of unbinding an unwanted shortcut
    };

    "xfce4-panel/launcher" = {
      # Optional: Pin favorite apps (replace with your .desktop entries)
      "items" = [
        "de.qitech.control-electron.desktop"
      ];
    };

    "xfce4-notifyd" = {
      # Enable notifications, animations
      "do-slideout" = true;
    };
  };

}
