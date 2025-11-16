#!/bin/bash
set -e

# SIA Uninstallation Script

INSTALL_DIR="/usr/local/bin"
SERVICE_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/sia"
DATA_DIR="/var/lib/sia"
SOCKET_DIR="/run/sia"

echo "🗑️  Uninstalling SIA (System Insight Agent)..."

# Check if running as root/sudo
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run with sudo: sudo ./uninstall.sh"
    exit 1
fi

# Stop and disable service
if systemctl is-active --quiet sia-agent; then
    echo "🛑 Stopping sia-agent service..."
    systemctl stop sia-agent
fi

if systemctl is-enabled --quiet sia-agent 2>/dev/null; then
    echo "🔌 Disabling sia-agent service..."
    systemctl disable sia-agent
fi

# Remove service file
if [ -f "$SERVICE_DIR/sia-agent.service" ]; then
    echo "🗑️  Removing systemd service..."
    rm -f "$SERVICE_DIR/sia-agent.service"
    systemctl daemon-reload
fi

# Remove binaries
echo "🗑️  Removing binaries..."
rm -f "$INSTALL_DIR/sia-agent"
rm -f "$INSTALL_DIR/sia-cli"

# Remove TypeScript CLI files
if [ -d "/usr/lib/sia/cli-ts" ]; then
    echo "🗑️  Removing TypeScript CLI..."
    rm -rf /usr/lib/sia/cli-ts
fi

# Ask before removing data
echo ""
read -p "❓ Remove configuration and data? ($CONFIG_DIR, $DATA_DIR) [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  Removing configuration and data..."
    rm -rf "$CONFIG_DIR"
    rm -rf "$DATA_DIR"
else
    echo "ℹ️  Keeping configuration and data"
fi

# Clean up socket directory (runtime only)
if [ -d "$SOCKET_DIR" ]; then
    rm -rf "$SOCKET_DIR"
fi

# Remove sia user if it exists
if id -u sia >/dev/null 2>&1; then
    echo "👤 Removing sia system user..."
    userdel sia
fi

echo ""
echo "✅ Uninstallation complete!"
