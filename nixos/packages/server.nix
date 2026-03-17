{ lib, pkgs, pkg-config, libudev-zero, libpcap, craneLib }:

let
  # Calculate jobs outside the attr set for clarity
  envJobs = builtins.getEnv "CARGO_BUILD_JOBS";
  jobs = if envJobs != "" then envJobs else "4";

  commonArgs = {
    pname = "server";
    version = "1.0.0";
    strictDeps = true;

    nativeBuildInputs = [ pkg-config ];
    buildInputs = [ libpcap libudev-zero ];

    src = craneLib.cleanCargoSource ../..;

    # 1. Set Environment Variables as attributes
    CARGO_BUILD_JOBS = jobs;

    # 2. Use preBuild only for shell commands
    preBuild = ''
      export CARGO="taskset -c 0-3 cargo"
    '';

    cargoExtraArgs = "--features tracing-journald,io-uring --no-default-features";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in 
craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;
})