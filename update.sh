#!/bin/bash
set -e

# SIA Update Script
# Rebuilds and updates installed binaries without removing config/data

INSTALL_DIR="/usr/local/bin"

echo "🔄 Updating SIA (System Insight Agent)..."

# Check if running as root/sudo
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run with sudo: sudo ./update.sh"
    exit 1
fi

# Get the original user (who ran sudo)
ORIGINAL_USER="${SUDO_USER:-$USER}"

# Check if service is running
if systemctl is-active --quiet sia-agent; then
    echo "🛑 Stopping sia-agent service..."
    systemctl stop sia-agent
    SERVICE_WAS_RUNNING=true
else
    SERVICE_WAS_RUNNING=false
fi

# Build release binaries as the original user (not root)
echo "📦 Building release binaries..."
if [ -n "$SUDO_USER" ]; then
    # Running under sudo - build as the original user
    su - "$SUDO_USER" -c "cd '$PWD' && cargo build --release --workspace"
else
    # Running as root directly
    cargo build --release --workspace
fi

# Install binaries
echo "🔧 Installing updated binaries to $INSTALL_DIR..."
cp target/release/sia-agent "$INSTALL_DIR/"
cp target/release/sia-cli "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/sia-agent"
chmod +x "$INSTALL_DIR/sia-cli"

# Restart service if it was running
if [ "$SERVICE_WAS_RUNNING" = true ]; then
    echo "▶️  Starting sia-agent service..."
    systemctl start sia-agent
    sleep 1
    if systemctl is-active --quiet sia-agent; then
        echo "✅ Service started successfully"
    else
        echo "⚠️  Service may have failed to start. Check logs: sudo journalctl -u sia-agent -n 20"
    fi
fi

echo ""
echo "✅ Update complete!"
echo ""
echo "📋 You can now:"
echo "   - Check status:  sia-cli status"
echo "   - View logs:     sudo journalctl -u sia-agent -f"
echo ""
