{ lib
, fetchFromGitHub
, rustPlatform
, pkg-config
, libudev-zero
, libpcap
, commitHash ? ""
, rust-bin ? null
}:

let
  rustStable = if rust-bin != null then
    rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" "rust-analyzer" ];
      targets = [ "x86_64-unknown-linux-gnu" ];
    }
  else
    rustPlatform.rust.rustc;
    
  # Create a custom rustPlatform with stable
  customRustPlatform = rustPlatform // {
    rust = rustPlatform.rust // {
      rustc = rustStable;
      cargo = rustStable;
    };
  };
in

customRustPlatform.buildRustPackage rec {
  pname = "qitech-control-server";
  version = "0.1.0${if commitHash != "" then "-${builtins.substring 0 7 commitHash}" else ""}";

  src = if commitHash != "" then
    fetchFromGitHub {
      owner = "qitechgmbh";
      repo = "control";
      rev = commitHash;
      sha256 = lib.fakeSha256; # Replace with actual hash after first build attempt
      # sha256 = ""; # You'll need to replace this with the actual hash
    }
  else
    lib.cleanSource ../..;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      # You might need to add dependency hashes here if they're not in the registry
    };
  };

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ libpcap libudev-zero ];

  # Build only the server package
  buildAndTestSubdir = "server";

  # Reduce memory usage during build
  CARGO_BUILD_JOBS = "1";

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
    description = "QiTech Industries Control Software - Server Component";
    homepage = "https://qitech.de";
    platforms = platforms.linux;
  };
}
