{ lib, pkgs, pkg-config, libudev-zero, libpcap, craneLib }:

let
  commonArgs = {
    pname = "server";
    version = "1.0.0";
    strictDeps = true;

    nativeBuildInputs = [ pkg-config ];
    buildInputs = [ libpcap libudev-zero ];

    src = craneLib.cleanCargoSource ../..;

    preBuild = ''
      export CARGO="taskset -c 0-3 cargo"

    CARGO_BUILD_JOBS =
      if (builtins.tryEval (builtins.getEnv "CARGO_BUILD_JOBS")).success then
        builtins.getEnv "CARGO_BUILD_JOBS"
      else
        "4";

    cargoExtraArgs =
      "--features tracing-journald,io-uring --no-default-features";
    '';
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in craneLib.buildPackage (commonArgs // {

  inherit cargoArtifacts;
})
