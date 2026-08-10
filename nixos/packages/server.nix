{
  lib,
  pkg-config,
  libudev-zero,
  libpcap,
  openblas,
  rustPlatform,
  # Iterated-LMI MIMO synthesis. Off by default because its semidefinite solver needs
  # BLAS/LAPACK, which is a C/Fortran dependency the rest of the server does without.
  withMimoLmi ? false,
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
  ] ++ lib.optional withMimoLmi openblas;

  doCheck = false;

  CARGO_BUILD_JOBS =
    if (builtins.tryEval (builtins.getEnv "CARGO_BUILD_JOBS")).success then
      builtins.getEnv "CARGO_BUILD_JOBS"
    else
      "2";

  cargoExtraArgs =
    "--features io-uring${lib.optionalString withMimoLmi ",mimo-lmi"} --no-default-features";
})
