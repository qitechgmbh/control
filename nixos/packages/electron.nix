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
    
    # Setup git
    mkdir -p $HOME
    ${git}/bin/git config --global --add safe.directory '*'
    ${git}/bin/git config --global user.email "nixbuild@localhost"
    ${git}/bin/git config --global user.name "Nix Builder"
    
    # Install dependencies
    echo "Installing dependencies..."
    npm ci --no-audit --no-fund || npm install --no-audit --no-fund --legacy-peer-deps
    
    # Build the Vite application first (important step!)
    echo "Building Vite app..."
    NODE_ENV=production npx vite build
    
    # Now package the app with the built files
    echo "Packaging application..."
    npm run package
    
    # Make distributables
    echo "Creating distributables..."
    npm run make || true
  '';

  installPhase = ''
    mkdir -p $out/share/qitech-electron $out/bin
    
    # Find packaged app
    PACKAGED_APP=$(find out -type d -name "*linux-x64" | head -n 1)
    
    if [ -n "$PACKAGED_APP" ]; then
      echo "Found packaged app at $PACKAGED_APP"
      cp -r $PACKAGED_APP/* $out/share/qitech-electron/
    else
      echo "No packaged app found, using build directory"
      # Copy the core built files
      mkdir -p $out/share/qitech-electron/.vite
      if [ -d ".vite/build" ]; then
        cp -r .vite/build $out/share/qitech-electron/.vite/
      fi
      
      # Copy other necessary files
      cp -r node_modules $out/share/qitech-electron/
      cp package.json $out/share/qitech-electron/
      
      # Copy assets and dist folder if they exist
      if [ -d "dist" ]; then
        cp -r dist $out/share/qitech-electron/
      fi
      if [ -d "assets" ]; then
        cp -r assets $out/share/qitech-electron/
      fi
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
    makeWrapper ${electron}/bin/electron $out/bin/qitech-electron \
      --add-flags "$out/share/qitech-electron" \
      --add-flags "--no-sandbox" \
      --set ELECTRON_ENABLE_LOGGING true
  '';

  meta = with lib; {
    description = "QiTech Industries Control Software - Electron Frontend";
    homepage = "https://qitech.com";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
