#!/bin/bash
set -e  # Exit immediately if a command fails

SERVICE_NAME="doggo-display"
BIN_NAME="doggo-display"
TARGET_DIR="/usr/local/bin"
CONFIG_DIR="/etc/doggo-display"
LOG_DIR="/var/log/doggo-display"
SYSTEMD_DIR="/etc/systemd/system"

echo "🚀 Setting up $SERVICE_NAME..."

# 1️⃣ Create necessary directories if they don't exist
echo "📁 Ensuring necessary directories exist..."
sudo mkdir -p "$CONFIG_DIR" "$LOG_DIR" "$SYSTEMD_DIR"

# 2️⃣ Move binary to target directory
echo "🚚 Moving binary to $TARGET_DIR"
sudo mv "$BIN_NAME" "$TARGET_DIR/$BIN_NAME"
sudo chmod +x "$TARGET_DIR/$BIN_NAME"

# 3️⃣ Install systemd service file
echo "📂 Installing systemd service..."
sudo mv "systemd/$SERVICE_NAME.service" "$SYSTEMD_DIR/$SERVICE_NAME.service"
sudo chmod 644 "$SYSTEMD_DIR/$SERVICE_NAME.service"

# 4️⃣ Copy config.toml if it doesn’t already exist
if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    echo "📜 Copying default config.toml"
    sudo mv "config.toml" "$CONFIG_DIR/config.toml"
else
    echo "⚠️ config.toml already exists, skipping copy."
fi

# 5️⃣ Reload systemd, enable, and start the service
echo "🛠️ Enabling and starting service..."
sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"
sudo systemctl restart "$SERVICE_NAME"

echo "✅ Installation complete! $SERVICE_NAME is running."
