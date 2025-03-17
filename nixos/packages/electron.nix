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
    
    # Install the app - check various output directories
    if [ -d "out" ]; then
      cp -r out/* $out/share/qitech-electron/ || true
    elif [ -d "dist" ]; then
      cp -r dist/* $out/share/qitech-electron/ || true
    else
      # Fallback - copy the source and node_modules
      echo "No build output found - using source files"
      cp -r src package.json $out/share/qitech-electron/ || true
      if [ -d "node_modules" ]; then
        cp -r node_modules $out/share/qitech-electron/ || true
      fi
    fi
    
    # Create desktop entry
    mkdir -p $out/share/applications
    cat > $out/share/applications/qitech-electron.desktop << EOF
    [Desktop Entry]
    Name=QiTech Control
    Comment=QiTech Industries Control Software
    Exec=qitech-electron
    Icon=$out/share/qitech-electron/icon.png
    Terminal=false
    Type=Application
    Categories=Development;Engineering;
    EOF
    
    # Find an icon if it exists
    if [ -f "build/icon.png" ]; then
      cp build/icon.png $out/share/qitech-electron/ || true
    fi
    
    # Create wrapper script
    makeWrapper ${electron}/bin/electron $out/bin/qitech-electron \
      --add-flags "$out/share/qitech-electron" \
      --add-flags "--no-sandbox"
  '';

  meta = with lib; {
    description = "QiTech Industries Control Software - Electron Frontend";
    homepage = "https://qitech.com";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
