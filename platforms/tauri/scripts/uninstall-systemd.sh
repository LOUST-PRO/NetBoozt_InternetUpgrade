#!/bin/bash
# uninstall-systemd.sh — Remove the NetBoozt headless DNS failover systemd user service.
#
# Usage: ./uninstall-systemd.sh
#
# By LOUST (www.loust.pro)

set -e

SERVICE_FILE="${HOME}/.config/systemd/user/netboozt-dns.service"

echo "==> NetBoozt DNS Failover — systemd user service uninstaller"

# Stop and disable
echo "==> Stopping netboozt-dns.service ..."
systemctl --user stop netboozt-dns.service 2>/dev/null || true

echo "==> Disabling netboozt-dns.service ..."
systemctl --user disable netboozt-dns.service 2>/dev/null || true

# Reload systemd
echo "==> Reloading systemd user daemon ..."
systemctl --user daemon-reload

# Remove unit file
if [[ -f "$SERVICE_FILE" ]]; then
    echo "Removing $SERVICE_FILE ..."
    rm -f "$SERVICE_FILE"
else
    echo "Service file not found at $SERVICE_FILE (already removed or never installed)"
fi

echo ""
echo "✅ netboozt-dns.service uninstalled."
echo ""
echo "Note: The NetBoozt binary at ~/.local/bin/NetBoozt was not modified."
