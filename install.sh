#!/bin/bash
set -euo pipefail

cargo build --release
sudo cp target/release/fast-mkinitcpio /opt/fast-mkinitcpio
