#!/bin/bash

# Define variables
SERVICE_NAME="adhanapp"
USERNAME=$(whoami)
BINARY_URL="https://github.com/ahmedjama/adhanapp/releases/download/v1.0.0/adhanapp-x86_64-unknown-linux-gnu"
SERVICE_PATH="$HOME/.config/systemd/user/$SERVICE_NAME.service"

# Check if root
if [[ $EUID -eq 0 ]]; then
    echo "This script should be run as a non-root user."
    exit 1
fi

# Define the folder paths
folders=(
  "$HOME/adhanapp/media"
  "$HOME/adhanapp/config"
  "$HOME/adhanapp/media/fajr"
  "$HOME/adhanapp/media/other"
)

# Loop through each folder path and create it if it doesn't exist
for folder in "${folders[@]}"; do
  if [ ! -d "$folder" ]; then
    mkdir -p "$folder"
    echo "Folder '$folder' created successfully!"
  else
    echo "Folder '$folder' already exists."
  fi
done

# Define the encoded API key
encoded_api_key="MmE5OWYxODktNmUzYi00MDE1LThmYjgtZmYyNzc2NDI1NjFk"

# Decode the API key
decoded_api_key=$(echo "$encoded_api_key" | base64 -d)

# Define the content for adhan_api.json
api_content=$(printf '{"api_key": "%s"}' "$decoded_api_key")

# Write the content to adhan_api.json (overwrite if exists)
api_file="$HOME/adhanapp/config/adhan_api.json"
echo "$api_content" | jq . > "$api_file"
echo "File '$api_file' created successfully!"

# Download the binary
binary_file="$HOME/bin/adhanapp"

# Create bin directory if it doesn't exist
mkdir -p "$HOME/bin"

# Download the binary to ~/bin and make it executable
curl -L -o "$binary_file" "$BINARY_URL"
chmod +x "$binary_file"
echo "Binary downloaded to '$binary_file'."

# Create the systemd service file
mkdir -p "$HOME/.config/systemd/user/"
cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=AdhanApp Service
After=network.target

[Service]
ExecStart=$binary_file
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF

# Reload systemd user units and start the service
systemctl --user daemon-reload
systemctl --user enable "$SERVICE_NAME"
systemctl --user start "$SERVICE_NAME"

echo "adhanapp installed and started successfully."
