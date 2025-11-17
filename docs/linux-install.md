# Installation Guide for Qitech-Control Software on Linux

This guide assumes you have an already installed linux distribution like debian,fedora etc.
It needs to a kernel that either has PREEMPT_RT patched or which we heavily recommend a mainline kernel later that 6.12, as versions afterwards have PREEMPT_RT integrated and can be switched with a kernel parameter.

Additionally it is assumed that at least four cpu cores are usable.

## Kernel Parameters
For our Ethercat control to run determinstically while also being as fast as possible you will need to configure a few kernel parameters.
Most Distributions use something like grub to persistently configure kernel parameters.

For Grub add the kernel parameters to the LINUX_COMMANDLINE_DEFAULT:

```

```

isolcpus=2,3 isolates the third and fourth core from the system scheduler, cutting down latency
nohz_full=2,3 isolates third anf fourth core from kernel interrupts
rcu_nocbs=2,3  
