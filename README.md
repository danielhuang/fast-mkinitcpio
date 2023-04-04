## fast-mkinitcpio

Runs `mkinitcpio` in parallel

## Installation

Run `./install.sh`, then edit `/usr/share/libalpm/hooks/90-mkinitcpio-install.hook` to replace `Exec = /usr/share/libalpm/scripts/mkinitcpio` with `Exec = /opt/fast-mkinitcpio`.
