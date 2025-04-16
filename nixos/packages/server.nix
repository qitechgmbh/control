{ lib
, stdenv
, pkg-config
, libudev-zero
, libpcap
, rust
}:

stdenv.mkDerivation rec {
  pname = "qitech-control-server";
  version = "0.1.0";

  src = lib.cleanSource ../..;

  nativeBuildInputs = [ pkg-config rust ];
  buildInputs = [ libpcap libudev-zero ];

  buildPhase = ''
    export HOME=$TMPDIR
    export CARGO_HOME=$TMPDIR/.cargo
    
    # Reduce memory usage
    export CARGO_BUILD_JOBS=1
    
    # Create a swap file if building on a memory-constrained system
    if [ ! -f /swapfile ] && [ $(free -m | grep Mem | awk '{print $2}') -lt 8000 ]; then
      mkdir -p $HOME/swap
      dd if=/dev/zero of=$HOME/swap/swapfile bs=1M count=4096
      chmod 600 $HOME/swap/swapfile
      mkswap $HOME/swap/swapfile
      swapon $HOME/swap/swapfile
    fi
    
    # Build with fewer parallel jobs
    ${rust}/bin/cargo build --release --package server
    
    # Cleanup swap if we created it
    if [ -f $HOME/swap/swapfile ]; then
      swapoff $HOME/swap/swapfile
      rm $HOME/swap/swapfile
    fi
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/server $out/bin/qitech-control-server
  '';

  meta = with lib; {
    description = "QiTech Industries Control Software - Server Component";
    homepage = "https://qitech.com";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
