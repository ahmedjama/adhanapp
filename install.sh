#!/bin/bash

# Define variables
SERVICE_NAME="adhanapp"
USERNAME=$(whoami)
BINARY_URL="https://github.com/ahmedjama/adhan/releases/latest/download/adhanapp"
SERVICE_PATH="/etc/systemd/system/$SERVICE_NAME.service"

# Check if root
if [[ $EUID -eq 0 ]]; then
    echo "This script should be run as a non-root user."
    exit 1
fi

# Check if wget is installed
if ! command -v wget &> /dev/null; then
    echo "wget is required to download the binary. Please install wget."
    exit 1
fi

# Download the precompiled binary
mkdir -p /home/$USERNAME/bin
wget -O /home/$USERNAME/bin/adhanapp $BINARY_URL
chmod +x /home/$USERNAME/bin/adhanapp

# Check if the binary was downloaded successfully
if [ ! -f "/home/$USERNAME/bin/adhanapp" ]; then
    echo "Failed to download the binary."
    exit 1
fi

# Create the systemd service file
cat > $SERVICE_PATH <<EOF
[Unit]
Description=AdhanApp Service
After=network.target

[Service]
User=$USERNAME
Group=$USERNAME
ExecStart=/home/$USERNAME/bin/adhanapp
Restart=always
RestartSec=3
StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=$SERVICE_NAME

[Install]
WantedBy=default.target
EOF

# Enable and start the service
systemctl --user daemon-reload
systemctl --user enable $SERVICE_NAME
systemctl --user start $SERVICE_NAME

echo "adhanapp installed and started successfully."
