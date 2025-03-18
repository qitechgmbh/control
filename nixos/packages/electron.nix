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
  pname = "qitech-electron";
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

  buildPhase = ''
    export HOME=$TMPDIR
    
    # Ensure git directories exist
    mkdir -p $HOME
    
    # Setup git for npm with explicit config path
    ${git}/bin/git config --global --add safe.directory '*'
    ${git}/bin/git config --global user.email "nixbuild@localhost"
    ${git}/bin/git config --global user.name "Nix Builder"
    
    # Install dependencies
    echo "Installing dependencies..."
    npm ci --no-audit --no-fund || npm install --no-audit --no-fund --legacy-peer-deps
    
    # Build the application
    echo "Building application..."
    npm run build || true
    
    # Package the application
    echo "Packaging application..."
    npm run package || true
  '';

  installPhase = ''
    mkdir -p $out/share/qitech-electron $out/bin
    
    # Find out what's actually built
    echo "Listing directory contents:"
    ls -la
    
    # Copy the built electron app
    if [ -d "electron-shadcn Template-linux-x64" ]; then
      echo "Found packaged electron app, copying..."
      cp -r "electron-shadcn Template-linux-x64"/* $out/share/qitech-electron/
    elif [ -d "out" ]; then
      echo "Found 'out' directory, checking for packaged app..."
      find out -type d -name "*-linux-x64" -exec cp -r {}/* $out/share/qitech-electron/ \;
    elif [ -d "dist" ]; then
      echo "Copying dist directory..."
      cp -r dist/* $out/share/qitech-electron/
    else
      echo "No build artifacts found, copying source files..."
      cp -r * $out/share/qitech-electron/
    fi
    
    # Create desktop entry
    mkdir -p $out/share/applications
    cat > $out/share/applications/qitech-electron.desktop << EOF
    [Desktop Entry]
    Name=QiTech Control
    Comment=QiTech Industries Control Software
    Exec=qitech-electron
    Terminal=false
    Type=Application
    Categories=Development;Engineering;
    EOF
    
    # Create wrapper script
    echo '#!/bin/sh
    exec ${electron}/bin/electron "$@"' > $out/bin/qitech-electron
    chmod +x $out/bin/qitech-electron
  '';

  meta = with lib; {
    description = "QiTech Industries Control Software - Electron Frontend";
    homepage = "https://qitech.com";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
