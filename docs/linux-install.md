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
rcu_nocbs=2,3 isolates read copy update from the given cpu cores, you guessed it less latency
preempt="full" turns linux into a realtime capable OS. Offering deterministic Execution for programs

After you changed the kernel parameters you have to rebuild the initramfs and bootloader, with something like grub.

```
# On Most Systems you need this command
sudo update-grub
# Or this one
sudo update-grub2
```

If those commands dont work for you or you dont use grub then simply, 
look up for your given distro what you need to do to persist kernel parameters.

## Installation On Generic Linux

This applies to any linux distro, that is not explicitily covered by this guide.

### Download Rust-toolchain
The easiest way is through rustup, simply execute the script as you home user and for everything keep it at defaults

### Download Npm and Node
The exact commands depend on which distro you use, but for most npm should exist as a package you can download in the package manager
