#!/bin/bash
# install-systemd.sh — Install NetBoozt headless DNS failover as a systemd user service.
#
# Usage: ./install-systemd.sh
# Requires: systemd user session, NetBoozt binary at ~/.local/bin/NetBoozt
#
# By LOUST (www.loust.pro)

set -e

BIN_PATH="${HOME}/.local/bin/NetBoozt"
SERVICE_FILE="${HOME}/.config/systemd/user/netboozt-dns.service"

echo "==> NetBoozt DNS Failover — systemd user service installer"

# Check binary exists
if [[ ! -x "$BIN_PATH" ]]; then
    echo "ERROR: NetBoozt binary not found at $BIN_PATH"
    echo "Build it first: cargo build --release --bin netboozt-headless"
    exit 1
fi

# Check binary is actually the headless one (basic sanity)
if ! "$BIN_PATH" --help >/dev/null 2>&1; then
    echo "WARNING: Binary at $BIN_PATH does not respond to --help"
fi

# Ensure systemd user dir exists
mkdir -p "${HOME}/.config/systemd/user"

# Install unit file
echo "Installing $SERVICE_FILE ..."
cp "$(dirname "$0")/netboozt-dns.service" "$SERVICE_FILE"
chmod 644 "$SERVICE_FILE"

# Reload systemd, enable and start
echo "==> Reloading systemd user daemon..."
systemctl --user daemon-reload

echo "==> Enabling netboozt-dns.service ..."
systemctl --user enable netboozt-dns.service

echo "==> Starting netboozt-dns.service ..."
systemctl --user start netboozt-dns.service

# Verify
if systemctl --user is-active --quiet netboozt-dns.service; then
    echo ""
    echo "✅ netboozt-dns.service is active."
    echo ""
    echo "Useful commands:"
    echo "  systemctl --user status netboozt-dns.service   # view status"
    echo "  journalctl --user -u netboozt-dns.service    # view logs"
    echo "  systemctl --user stop netboozt-dns.service    # stop"
    echo "  systemctl --user restart netboozt-dns.service # restart"
else
    echo ""
    echo "⚠️  Service started but may not be fully active yet. Check:"
    echo "  systemctl --user status netboozt-dns.service"
    echo "  journalctl --user -u netboozt-dns.service"
fi
