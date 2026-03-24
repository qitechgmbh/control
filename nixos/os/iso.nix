{
  modulesPath,
  lib,
  pkgs,
  ...
}:
let
  installScript = pkgs.writeShellScriptBin "qitech-setup" (builtins.readFile ../../nixos-setup.sh);
  installScriptDesktopFile = pkgs.makeDesktopItem {
    name = "qitech-setup";
    exec = "${lib.getExe pkgs.gnome-console} -e ${lib.getExe installScript}";
    icon = ../../electron/src/assets/icon.png;
    comment = "Integrated Development Environment";
    desktopName = "Install QiTech Control";
  };
in
{
  imports = [
    ./configuration.nix
    "${toString modulesPath}/installer/cd-dvd/installation-cd-base.nix"
  ];

  environment = {
    variables = {
      # Fix calamares fractional scaling
      QT_QPA_PLATFORM = "$([[ $XDG_SESSION_TYPE = \"wayland\" ]] && echo \"wayland\")";
    };
    defaultPackages = with pkgs; [
      gparted
      vim
      nano
      # Calamares for graphical installation
      calamares-nixos
      calamares-nixos-extensions
      glibcLocales
      # Our installer
      installScript
      installScriptDesktopFile
    ];
  };

  i18n.supportedLocales = [ "all" ];
}
