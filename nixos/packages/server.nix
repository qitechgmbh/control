{ lib
, pkgs
, rustPlatform
, pkg-config
, libudev-zero
, libpcap
}:

rustPlatform.buildRustPackage {
  pname = "server";
  version = "1.0.0";

  src = pkgs.lib.cleanSource ../..;

  cargoLock.lockFile = ../.. + "/Cargo.lock";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ libpcap libudev-zero ];

  cargoBuildFlags = [
    "--package" "server"
    "--features" "tracing-journald,io-uring"
    "--no-default-features"
  ];

  # Create a swap file if building on a memory-constrained system
  preBuild = ''
      if [ $(free -m | grep Mem | awk '{print $2}') -lt 6000 ]; then
        mkdir -p $TMPDIR/swap
        dd if=/dev/zero of=$TMPDIR/swap/swapfile bs=1M count=8192
        chmod 600 $TMPDIR/swap/swapfile
        mkswap $TMPDIR/swap/swapfile
        swapon $TMPDIR/swap/swapfile
      fi
  '';
  CARGO_BUILD_JOBS = "2";

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
}
