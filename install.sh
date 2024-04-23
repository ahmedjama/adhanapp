#!/bin/bash

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


# Download the binary
binary_url="https://github.com/ahmedjama/adhanapp/releases/download/v1.0.0/adhanapp-x86_64-unknown-linux-gnu"
binary_file="$HOME/bin/adhanapp"

# Create bin directory if it doesn't exist
mkdir -p "$HOME/bin"


curl -L -o "$binary_file" "$binary_url"
chmod +x "$binary_file"
echo "Binary downloaded to '$binary_file'."

# Create the systemd service file
cat > "$HOME/.config/systemd/user/adhanapp.service" <<EOF
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
systemctl --user enable adhanapp
systemctl --user start adhanapp

echo "adhanapp installed and started successfully."

# Download the Fajr adhan MP3 file if it doesn't exist
fajr_mp3_path="$HOME/adhanapp/media/fajr/adhan-fajr.mp3"
if [ ! -f "$fajr_mp3_path" ]; then
  cd "$HOME/adhanapp/media/fajr" || exit
  wget https://github.com/ahmedjama/adhanapp/raw/main/media/fajr/adhan-fajr.mp3
  echo "Fajr adhan MP3 downloaded to '$fajr_mp3_path'."
else
  echo "Fajr adhan MP3 already exists. Skipping download."
fi

# Download the other adhan MP3 file if it doesn't exist
other_mp3_path="$HOME/adhanapp/media/other/adhan-makkah2-dua.mp3"
if [ ! -f "$other_mp3_path" ]; then
  cd "$HOME/adhanapp/media/other" || exit
  wget https://github.com/ahmedjama/adhanapp/raw/main/media/other/adhan-makkah2-dua.mp3
  echo "Other adhan MP3 downloaded to '$other_mp3_path'."
else
  echo "Other adhan MP3 already exists. Skipping download."
fi
