{ lib, pkgs, pkg-config, libudev-zero, libpcap, craneLib }:

let
  # Bind variables so they can be inherited inside inner calls
  pname = "server";
  version = "1.0.0";
  strictDeps = true;
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ libpcap libudev-zero ];

  # Cleaned source
  src = craneLib.cleanCargoSource ../..;

  # Safe fallback for CARGO_BUILD_JOBS in impure builds
  cargoJobs =
    if (builtins.tryEval (builtins.getEnv "CARGO_BUILD_JOBS")).success then
      builtins.getEnv "CARGO_BUILD_JOBS"
    else
      "2";

in craneLib.buildPackage {
  inherit pname version src strictDeps nativeBuildInputs buildInputs;

  # Use the safe cargoJobs variable
  CARGO_BUILD_JOBS = cargoJobs;

  # Build cargo dependencies once for caching
  cargoArtifacts = craneLib.buildDepsOnly {
    inherit src strictDeps nativeBuildInputs buildInputs pname version;
  };

  cargoExtraArgs =
    "-p server --features tracing-journald,io-uring --no-default-features";
}
