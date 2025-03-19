{ lib
, stdenv
, makeWrapper
, nodejs
, electron
, nodePackages
, git
, cacert
}:

stdenv.mkDerivation rec {
  pname = "qitech-control-electron";
  version = "0.1.0";

  src = lib.cleanSource ../../electron;

  nativeBuildInputs = [
    nodejs
    nodePackages.npm
    git
    makeWrapper
    cacert
  ];

  # Environment variables for the build
  SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";
  GIT_SSL_CAINFO = "${cacert}/etc/ssl/certs/ca-bundle.crt";
  npm_config_update_notifier = false;
  npm_config_cache = "$TMPDIR/npm-cache";
  
  # Let Electron Forge see more output
  ELECTRON_ENABLE_LOGGING = "true";
  DEBUG = "*";

  buildPhase = ''
    export HOME=$TMPDIR
    
    # Setup git
    mkdir -p $HOME
    ${git}/bin/git config --global --add safe.directory '*'
    ${git}/bin/git config --global user.email "nixbuild@localhost"
    ${git}/bin/git config --global user.name "Nix Builder"
    
    # Install dependencies
    echo "Installing dependencies..."
    npm ci --no-audit --no-fund || npm install --no-audit --no-fund --legacy-peer-deps
    
    # First try the 'make' command which does a full build + package
    echo "Creating distributable..."
    NODE_ENV=production npm run make || {
      echo "Make failed, trying package instead..."
      NODE_ENV=production npm run package
    }
    
    # List the output directories to see what we got
    echo "Created artifacts:"
    find out -type d | sort
    find out -type f -name "*.js" | sort
  '';

  installPhase = ''
    mkdir -p $out/share/qitech-control-electron $out/bin $out/share/applications $out/share/icons/hicolor/256x256/apps
    
    # First check for a full distribution in the 'out/make' directory
    if [ -d "out/make" ]; then
      echo "Using distribution from 'out/make'"
      find out/make -type d -name "*linux*" -exec cp -r {} $out/share/qitech-control-electron \; || true
    fi
    
    # If no distribution was found, use the packaged version
    if [ ! "$(ls -A $out/share/qitech-control-electron)" ]; then
      echo "Using packaged app from 'out'"
      find out -maxdepth 1 -type d -name "*linux-x64" -exec cp -r {}/* $out/share/qitech-control-electron \; || true
    fi
    
    # If still nothing, try the Electron Forge template approach
    if [ ! "$(ls -A $out/share/qitech-control-electron)" ]; then
      echo "No packaged app found, using source + .vite directory"
      cp -r . $out/share/qitech-control-electron/
    fi
    
    # Copy the icon
    if [ -f "src/assets/icon.png" ]; then
      echo "Copying icon from src/assets/icon.png"
      cp src/assets/icon.png $out/share/icons/hicolor/256x256/apps/de.qitech.control-electron.png
      
      # Also create a symlink with the application name for compatibility
      mkdir -p $out/share/pixmaps
      ln -sf $out/share/icons/hicolor/256x256/apps/de.qitech.control-electron.png $out/share/pixmaps/qitech-control-electron.png
    fi

    # Create the executable wrapper with proper app ID
    cat > $out/bin/qitech-control-electron << EOF
    #!/bin/sh
    cd $out/share/qitech-control-electron
    exec ${electron}/bin/electron "$out/share/qitech-control-electron" \
      --no-sandbox \
      --class=de.qitech.control-electron \
      --name="QiTech Control" \
      --single-instance \
      --winrm-class="de.qitech.control-electron" \
      "\$@"
    EOF
    chmod +x $out/bin/qitech-control-electron

    # Create desktop entry with consistent application ID
    cat > $out/share/applications/de.qitech.control-electron.desktop << EOF
    [Desktop Entry]
    Type=Application
    Name=QiTech Control
    Comment=QiTech Industries Control Software
    Exec=env QITECH_BUILD_ENV=control-os QITECH_DEPLOYMENT_TYPE=production qitech-control-electron %U
    Icon=de.qitech.control-electron
    Terminal=false
    StartupWMClass=qitech-control-electron
    Categories=Development;Engineering;
    X-GNOME-UsesNotifications=true
    EOF
  '';

  meta = with lib; {
    description = "QiTech Industries Control Software - Electron Frontend";
    homepage = "https://qitech.com";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
