{ lib
, pkgs
, pkg-config
, libudev-zero
, libpcap
, commitHash
, crane
}:

let
  # Create crane lib properly
  craneLib = crane.mkLib pkgs;
  
  # Use crane's source cleaning which is more intelligent for Cargo projects
  src = craneLib.cleanCargoSource ../..;
  
  # Common arguments for both dependency and app builds
  commonArgs = {
    inherit src;
    strictDeps = true;
    
    nativeBuildInputs = [ pkg-config ];
    buildInputs = [ libpcap libudev-zero ];
    
    # Build only the server package with journald logging for NixOS
    pname = "server";
    version = commitHash;
    
    # Reduce memory usage during build
    CARGO_BUILD_JOBS = "1";
  };
  
  # Build *just* the cargo dependencies (of the entire workspace),
  # so we can reuse all of that work when running in CI
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

in
# Uses Rust 1.86 stable from nixpkgs 25.05 with Crane for dependency caching
craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;
  
  # Enable journald logging feature for NixOS systems and build only server package
  cargoExtraArgs = "-p server --features tracing-journald --no-default-features";

  # Create a swap file if building on a memory-constrained system
  preBuild = ''
      if [ $(free -m | grep Mem | awk '{print $2}') -lt 8000 ]; then
        mkdir -p $TMPDIR/swap
        dd if=/dev/zero of=$TMPDIR/swap/swapfile bs=1M count=4096
        chmod 600 $TMPDIR/swap/swapfile
        mkswap $TMPDIR/swap/swapfile
        swapon $TMPDIR/swap/swapfile
      fi
  '';

  postBuild = ''
    if [ -f $TMPDIR/swap/swapfile ]; then
      swapoff $TMPDIR/swap/swapfile
      rm $TMPDIR/swap/swapfile
    fi
  '';

  meta = with lib; {
    description = "QiTech Control Server";
    homepage = "https://qitech.de";
    platforms = platforms.linux;
  };
})
