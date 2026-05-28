{
  pkg-config,
  libudev-zero,
  libpcap,
  craneLib,
}:

let
  commonArgs = {
    pname = "server";
    version = (builtins.fromJSON (builtins.readFile ../../electron/package.json)).version;
    strictDeps = true;

    nativeBuildInputs = [ pkg-config ];

    buildInputs = [
      libpcap
      libudev-zero
    ];

    src = ../..;

    CARGO_BUILD_JOBS =
      if (builtins.tryEval (builtins.getEnv "CARGO_BUILD_JOBS")).success then
        builtins.getEnv "CARGO_BUILD_JOBS"
      else
        "2";

    cargoExtraArgs = "--features io-uring --no-default-features";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;
  }
)
