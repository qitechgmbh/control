# Performance/Stability Tweaks 
## EtherCrab Config
We set ethercrab to default settings for everything except for RetryBehaviour, so that one unlikely failure doesnt destroy the EtherCAT network.

```rust
    let maindevice = MainDevice::new(
        pdu,
        Timeouts {
            // Default 5000ms
            state_transition: Duration::from_millis(5000),
            // Default 30_000us
            pdu: Duration::from_micros(30_000),
            // Default 10ms
            eeprom: Duration::from_millis(10),
            // Default 0ms
            wait_loop_delay: Duration::from_millis(0),
            // Default 100ms
            mailbox_echo: Duration::from_millis(100),
            // Default 1000ms
            mailbox_response: Duration::from_millis(1000),
        },
        MainDeviceConfig {
            // Default RetryBehaviour::None
            retry_behaviour: RetryBehaviour::Count(5),
            // Default 10_000
            dc_static_sync_iterations: 10_000,
        },
    );

```
wait_loop_delay CAN in theory also decrease CPU usage if set exactly as the loop_delay we set, however this setting has caused us some issues, so we just leave it as is.

## Realtime Kernel
In our nixOs Config we use the latest realtime kernel and isolate TWO cores out of the Four available for the machines we use.

```nix
 #From configuration.nix
 boot.loader.efi.canTouchEfiVariables = true;
 boot.kernelPackages = pkgs.linuxPackages-rt_latest;
 boot.kernelModules = [ "i915" ];
```

Notice how we hardcode the intel gpu kernel module. This is because only the GPU driver gets blacklisted for some reason. By specifying it manually it loads like before.

One Core is used for the EtherCrab TXRX Thread and one is used for the machine Thread. 
```
    "isolcpus=2,3" # Isolate cpus 2 and 3 from scheduler for better latency, 2 runs ethercatthread and 3 runs server control-loop
    "nohz_full=2,3" # In this mode, the periodic scheduler tick is stopped when only one task is running, reducing kernel interruptions on those CPUs.
    "rcu_nocbs=2,3" # Moves RCU (Read-Copy Update) callback processing away from CPUs 2 and 3.
```

Essentially we completely disable acces from the operating system to these two cpus, which saves scheduler overhead, which would introduce noticable latency.
Its not entirely clear yet if what we did here is overkill. Maybe we could use only one core and get away with that?
Processor load normally hovers around 18% for one of the cores and like 9% for the other.

We also have some minor cpu tweaks in configuration.nix
```
    # Graphical
    "logo.nologo" # Remove kernel logo during boot

    # Performance

    # Specific Vulnerabilities Addressed by Mitigations:
    # - Spectre variants (V1, V2, V4, SWAPGS, SpectreRSB, etc.)
    # - Meltdown (Rogue Data Cache Load)
    # - Foreshadow/L1TF (L1 Terminal Fault)
    # - Microarchitectural Data Sampling (MDS, RIDL, Fallout, ZombieLoad)
    # - SRBDS (Special Register Buffer Data Sampling)
    # - TSX Asynchronous Abort (TAA)
    # - iTLB Multihit
    # - And others as they're discovered and mitigated
    #
    # With mitigations=off
    # - PROS: Maximum performance, equivalent to pre-2018 behavior
    # - CONS: Vulnerable to Spectre, Meltdown, Foreshadow, ZombieLoad, etc.
    #         Should ONLY be used in completely trusted environments
    # - Improves performance by 7-43%
    "mitigation=off"
    "intel_pstate=performance"    # Intel CPU-specific performance mode (if applicable)

    # Memory Management
    "transparent_hugepage=always" # Use larger memory pages for memory intense applications
    "nmi_watchdog=0"              # Disable NMI watchdog for reduced CPU overhead and realtime execution

    # High-throughput ethernet parameters
    "pcie_aspm=off"         # Disable PCIe power management for NICs
    "intel_iommu=off"       # Disable IOMMU (performance gain)

    # Reliability
    "panic=10"              # Auto-reboot 10 seconds after kernel panic
    "oops=panic"            # Treat kernel oops as panic for auto-recovery
    "usbcore.autosuspend=-1"     # Possibly fixes dre disconnect issue?
```

We should check if its really required to turn mitigations off as well.

## Rust Code Optimizations

In General we try to avoid memory allocations in our loop as much as possible, as it is not deterministic

One problem we found, is that due to the realtime kernel our ethernet driver caused us quite large spikes in our tx_rx cycle time for EtherCAT.
These spikes happen when CPU Load is very high on the two usable cores, causing the interrupt handler for Ethernet to stall.

Which is why we reassign the interrupt handler on startup to the same core that the EtherCatTxRx Thread runs on:

```rust
#[cfg(target_os = "linux")]
match set_irq_affinity(&interface, 3) {
      Ok(_) => tracing::info!("ethernet interrupt handler now runs on cpu:{}", 3),
      Err(e) => tracing::error!("set_irq_affinity failed: {:?}", e),
}
// Set core affinity to 4th core
let _ = set_core_affinity(3);

// Set the thread to real-time priority
let _ = set_realtime_priority();
```
This means that the Interrupt Handler ALWAYS has enough ressources to cope with our demand for realtime communication with EtherCAT


One Problem we have at the moment (we think) is the locking and async code blocking our server loop. 
Note that this doesnt affect the EtherCAT communication at all.

## Electron Optimizations (Not Yet Implemented)

Currently we are struggling with the fact, that apparently Electron doesnt really have good support for HW acceleration on Linux.
Electron runs with the software-renderer, which is VERY slow compared to a gpu. 
This would eliminate the weird stuttering seen on Graphs and make them run buttery smooth, while also alleviating load off of the CPU.