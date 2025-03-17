{ lib
, stdenv
, pkg-config
, libudev-zero
, libpcap
, rust-bin
}:

let
  rust = rust-bin.beta.latest.default;
in
stdenv.mkDerivation rec {
  pname = "qitech-server";
  version = "0.1.0";

  src = lib.cleanSource ../..;

  nativeBuildInputs = [
    pkg-config
    rust
  ];

  buildInputs = [
    libpcap
    libudev-zero
  ];

  buildPhase = ''
    export CARGO_HOME=$TMPDIR/.cargo
    cd server
    cargo build --release
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp ../target/release/server $out/bin/
  '';

  meta = with lib; {
    description = "QiTech Industries Control Software - Server Component";
    homepage = "https://qitech.com";  # Replace with actual homepage
    license = licenses.mit;  # Replace with actual license
    maintainers = [];
    platforms = platforms.linux;
  };
}
