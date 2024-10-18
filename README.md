## fast-mkinitcpio

Runs `mkinitcpio` in parallel. 

## Installation

Run `./install.sh`, then create `/etc/pacman.d/hooks/90-mkinitcpio-install.hook` with the following contents:

```
[Trigger]
Type = Path
Operation = Install
Operation = Upgrade
Operation = Remove
Target = usr/lib/initcpio/*
Target = usr/lib/firmware/*
Target = usr/src/*/dkms.conf
Target = usr/lib/systemd/systemd
Target = usr/bin/cryptsetup
Target = usr/bin/lvm

[Trigger]
Type = Path
Operation = Install
Operation = Upgrade
Target = usr/lib/modules/*/vmlinuz

[Trigger]
Type = Package
Operation = Install
Operation = Upgrade
Target = mkinitcpio
Target = mkinitcpio-git

[Action]
Description = Updating linux initcpios...
When = PostTransaction
Exec = /opt/fast-mkinitcpio
NeedsTargets
```