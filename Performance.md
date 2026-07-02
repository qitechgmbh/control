# Performance & Stability

The control loop has to hit a fixed cycle without jitter, so both the software and the operating system are tuned for real-time behaviour.

## The cycle

The backend targets a **1 ms cycle**. All devices are kept in lockstep by EtherCAT Distributed Clocks: the sync signal runs at the cycle period and is shifted by half a cycle, with a short start delay before synchronisation begins. Bus transfers use `io_uring`, and the machine loop polls the shared process image every 100 µs.

## Real-time threads

The EtherCAT bus thread runs at the highest real-time priority (99); its IO thread runs at a lower real-time priority (50). Both are pinned to an isolated CPU core, and the network interrupt is pinned to the same core, so bus traffic is not disturbed by other work on the system. The process also **locks its memory**, so the real-time path never has to wait for the kernel to page memory back in.

## Operating-system tuning (NixOS)

The device runs a fully preemptible real-time kernel (Linux 6.18, `preempt=full`). Two CPU cores (2 and 3) are taken out of the normal scheduler (`isolcpus`), run tickless (`nohz_full`), and have their RCU callbacks moved elsewhere (`rcu_nocbs`), leaving those cores almost entirely to the control software.

The service is granted only the capabilities it needs — real-time scheduling (`CAP_SYS_NICE`), memory locking (`CAP_IPC_LOCK`), and raw network access for EtherCAT (`CAP_NET_RAW`), among a few others.

Fault handling favours returning to a known state: the kernel reboots on an oops or shortly after a panic, swapping is kept low to avoid latency spikes, and emergency SysRq is enabled.

## Staying stable at runtime

Before going operational, the backend waits for the bus to report a stable pre-operational state across consecutive checks — with a timeout that forces a clean restart — rather than rushing into the operational state. A working-counter mismatch tolerance and a grace period while ramping up absorb brief bus hiccups instead of treating every glitch as fatal.

Each machine is updated independently. If one machine's update fails, that machine is reported and removed while the rest keep running, so a single faulty machine cannot take down the whole control loop.
