{ lib
, rustPlatform
, buildNpmPackage
, pkg-config
, openssl
, webkitgtk_4_1
, gtk3
, glib
, cairo
, pango
, gdk-pixbuf
, libsoup_3
, atk
}:

let
  frontend = buildNpmPackage {
    pname = "qitech-control-tauri-frontend";
    version = "1.0.0";

    srcs = ../../tauri;
    sourceRoot = "tauri";

    makeCacheWritable = true;
    npmDepsHash = lib.fakeHash;
    npmFlags = [ "--no-audit" "--no-fund" ];

    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp -rv dist/* $out/
      runHook postInstall
    '';
  };
in
rustPlatform.buildRustPackage {
  pname = "qitech-control-tauri";
  version = "2.15.0";

  src = ../../tauri/src-tauri;

  cargoHash = lib.fakeHash;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
    webkitgtk_4_1
    gtk3
    glib
    cairo
    pango
    gdk-pixbuf
    libsoup_3
    atk
  ];

  # Point Tauri to the pre-built frontend
  TAURI_FRONTEND_DIST = "${frontend}";

  postInstall = ''
    # desktop entry
    mkdir -p $out/share/applications
    cat > $out/share/applications/de.qitech.control.desktop << EOF
    [Desktop Entry]
    Type=Application
    Name=QiTech Control
    Comment=QiTech Control
    Exec=qitech-control-tauri %U
    Icon=de.qitech.control
    Terminal=false
    StartupWMClass=QiTech Control
    Categories=Development;Engineering;
    X-GNOME-UsesNotifications=true
    EOF
  '';

  meta = with lib; {
    description = "QiTech Control — Tauri shell";
    homepage = "https://qitech.de";
    platforms = platforms.linux;
  };
}
