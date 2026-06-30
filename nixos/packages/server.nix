{
  pkg-config,
  libudev-zero,
  libpcap,
  rustPlatform,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "server";
  version = (builtins.fromJSON (builtins.readFile ../../electron/package.json)).version;

  src = ../..;

  cargoLock = {
    lockFile = ../../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    libpcap
    libudev-zero
  ];

  CARGO_BUILD_JOBS =
    if (builtins.tryEval (builtins.getEnv "CARGO_BUILD_JOBS")).success then
      builtins.getEnv "CARGO_BUILD_JOBS"
    else
      "2";

  cargoExtraArgs = "--features io-uring --no-default-features";
})
