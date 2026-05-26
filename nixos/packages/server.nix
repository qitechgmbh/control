{
  lib,
  pkgs,
  pkg-config,
  libudev-zero,
  libpcap,
  craneLib,
}:

let
  runner = pkgs.writeShellScriptBin "run-linux" ''
    exec $@
  '';

  commonArgs = {
    pname = "server";
    version = "1.0.0";
    strictDeps = true;

    # Avoid using sudo as set by the cargo config
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="${runner}/bin/run-linux";

    nativeBuildInputs = [ pkg-config ];

    buildInputs = [
      libpcap
      libudev-zero
      runner
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
