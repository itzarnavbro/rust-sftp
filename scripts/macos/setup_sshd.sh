#!/usr/bin/env bash
set -euo pipefail
# Hinglish: Mac ko SSH server banate hain, aur authorized_keys ready karte hain.

# 1) SSH server (Remote Login) ON
#   Note: 'systemsetup' ko sudo chahiye. Dialog aa sakta hai.
sudo systemsetup -setremotelogin on

# 2) ~/.ssh/authorized_keys ensure + perms theek
mkdir -p "$HOME/.ssh"
chmod 700 "$HOME/.ssh"
touch "$HOME/.ssh/authorized_keys"
chmod 600 "$HOME/.ssh/authorized_keys"

# 3) Helpful: IP print kar do (LAN me yahi use hoga)
echo "Your LAN IPs:"
ipconfig getifaddr en0 || true
ipconfig getifaddr en1 || true
ipconfig getifaddr lo0 || true

echo "Done. Share your IP + mac username with your friend."
