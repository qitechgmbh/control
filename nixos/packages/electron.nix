{ lib, buildNpmPackage, electron }:

buildNpmPackage rec {
  pname = "qitech-control-electron";
  version = "1.0.0";

  srcs = [ ../../electron ../../docs ];
  sourceRoot = "electron";

  ELECTRON_SKIP_BINARY_DOWNLOAD = 1;

  makeCacheWritable = true;
  npmDepsHash = "sha256-gk/YC3+gc5DVvPS4ght9WmFcuc2FAX/UB+lIuQn436U=";
  npmFlags = [ "--no-audit" "--no-fund" ];

  installPhase = ''
    runHook preInstall

    mkdir -p $out/share/qitech-control-electron
    cp -rv dist/* $out/share/qitech-control-electron
    cp -rv dist-electron/* $out/share/qitech-control-electron
    cp -v src/assets/icon.png $out/share/qitech-control-electron

    if [ -f src/assets/icon.png ]; then
      mkdir -p $out/share/icons/hicolor/256x256/apps $out/share/pixmaps

      cp src/assets/icon.png $out/share/icons/hicolor/256x256/apps/de.qitech.control-electron.png
      ln -sf $out/share/icons/hicolor/256x256/apps/de.qitech.control-electron.png \
        $out/share/pixmaps/qitech-control-electron.png
    fi

    # wrapper
    mkdir -p $out/bin
    cat > $out/bin/qitech-control-electron << EOF
    #!/bin/sh
    appdir="$out/share/qitech-control-electron"
    cd "\$appdir"
    exec nice -n -20 ${electron}/bin/electron "\$appdir/main.js" \
      --no-sandbox \
      --class=de.qitech.control-electron \
      --name="QiTech Control" \
      --single-instance \
      --winrm-class="de.qitech.control-electron" \
      --enable-gpu \
      --ignore-gpu-blocklist \
      --enable-gpu-rasterization \
      --disable-software-rasterizer \
      "$@"
    EOF
    chmod +x $out/bin/qitech-control-electron

    # desktop entry
    mkdir -p $out/share/applications
    cat > $out/share/applications/de.qitech.control-electron.desktop << EOF
    [Desktop Entry]
    Type=Application
    Name=QiTech Control
    Comment=QiTech Control
    Exec=qitech-control-electron %U
    Icon=de.qitech.control-electron
    Terminal=false
    StartupWMClass=QiTech Control
    Categories=Development;Engineering;
    X-GNOME-UsesNotifications=true
    EOF
  '';

  meta = with lib; {
    description = "QiTech Control Electron";
    homepage = "https://qitech.de";
    platforms = platforms.linux;
  };
}
