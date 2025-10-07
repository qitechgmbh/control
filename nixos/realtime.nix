{ pkgs, ... }:

let
  lib = pkgs.lib;

  # 1) Point this to the exact RT patch that matches your kernel version.
  #    Example form (you must verify): 
  #    https://cdn.kernel.org/pub/linux/kernel/projects/rt/6.13/older/patch-6.13.1-rtX.patch.xz
  #    or sometimes .../rt/6.13/patch-6.13.y-rtZZ.patch.xz
  rtPatch = pkgs.fetchurl {
    url = "https://cdn.kernel.org/pub/linux/kernel/projects/rt/6.13/patch-6.13.y-rtZZ.patch.xz";
    sha256 = ""; # build once, copy the “got:” hash from the error, and put it here
  };

  # 2) Start from the stock 6.13 kernel derivation and layer in the patch + config.
  #    If your channel exposes linux_6_13 under a slightly different name, adjust it here.
  kernel-rt = pkgs.linux_6_13.override {
    # Apply the RT patch after any nixpkgs patches
    kernelPatches = lib.mkAfter [
      {
        name = "preempt-rt";
        patch = rtPatch;
        # Optionally force a tiny Kconfig fragment with the patch
        extraConfig = ''
          PREEMPT y
          PREEMPT_RT y
          # If the symbol name differs, use PREEMPT_RT_FULL or similar as required by your patch
        '';
      }
    ];

    # Enable RT in the .config (structured form is tidier and validated)
    structuredExtraConfig = with lib.kernel; {
      PREEMPT           = yes;
      PREEMPT_BUILD     = lib.mkDefault yes;
      PREEMPT_RT        = yes;  # some trees use PREEMPT_RT_FULL; if the build fails, check the symbol name
      HZ_1000           = yes;  # 1kHz timer is typical for RT
      HZ_250            = lib.mkForce no;
      HZ_300            = lib.mkForce no;
      HZ_100            = lib.mkForce no;

      # These can help avoid spurious build prompts
      EXPERT            = yes;
      SYSTEM_TRUSTED_KEYS = freeform "";   # avoid embedding kernel.org keys if you want a “clean” build
      SYSTEM_REVOCATION_KEYS = freeform "";
    };

    # If the patch series expects certain config defaults, you can allow unknowns:
    ignoreConfigErrors = true;
  };
in
  # 3) Turn that kernel into a kernelPackages set (so modules, ZFS, etc. match your kernel)
  pkgs.linuxPackagesFor kernel-rt
