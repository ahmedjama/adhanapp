# adhanapp

This repository contains code for an Adhan application written in Rust. The application fetches and processes prayer times from an API, plays the Adhan (Islamic call to prayer) at the appropriate times, and manages volume control for the media player.

## Dependencies
- `chrono`: For date and time manipulation.
- `std::thread`: For creating threads to run tasks concurrently.
- `home`: To determine the user's home directory.
- `rodio`: For audio playback.
- `serde`: For serializing and deserializing JSON data.
- `rand`: For random selection of audio files.
- `reqwest`: For making HTTP requests to fetch prayer times.

## Installation
1. Clone this repository to your local machine.
2. Ensure you have Rust installed. If not, install it from [Rust's official website](https://www.rust-lang.org/tools/install).
3. Run `cargo build` to build the application.

# Installation Script: install.sh

This script automates the installation process for the Adhan application on a Linux system. It creates necessary directories, downloads the application binary, sets up a systemd service, and downloads required audio files for Adhan.

## Usage
1. Ensure you are logged in as a non-root user.
2. Run the script using `./install.sh`.
3. Follow the on-screen prompts if any.

## Steps Performed by the Script

1. **Folder Creation**:
   - Creates the following directory structure:
     - `$HOME/adhanapp/media`
     - `$HOME/adhanapp/config`
     - `$HOME/adhanapp/media/fajr`
     - `$HOME/adhanapp/media/other`
   - Checks if the directories already exist before creating.

2. **Binary Download**:
   - Downloads the Adhan application binary from the provided URL.
   - Places the binary in `$HOME/bin/adhanapp`.
   - Sets executable permissions for the binary.

3. **Systemd Service Setup**:
   - Creates a systemd service file named `adhanapp.service` in `$HOME/.config/systemd/user/`.
   - Configures the service to start the Adhan application on system boot.

4. **Service Activation**:
   - Reloads systemd user units.
   - Enables the `adhanapp` service.
   - Starts the `adhanapp` service.

5. **Audio File Download**:
   - Downloads the Fajr Adhan MP3 file if it doesn't exist in `$HOME/adhanapp/media/fajr`.
   - Downloads the Other Adhan MP3 file if it doesn't exist in `$HOME/adhanapp/media/other`.

## Notes
- Ensure you have necessary permissions to execute the script and modify directories.
- This script assumes a Linux environment with systemd.
- Modify the script if the directory structure or URLs change.


## Features
- **Automatic Prayer Time Update**: The application automatically fetches and updates prayer times.
- **Flexible Configuration**: Customize the audio files for each prayer time by placing them in the appropriate directories.
- **Volume Control**: Control system volume during playback to ensure a pleasant experience.

## Configuration
- **API URL**: The application fetches prayer times from the provided API URL. Modify `fetch_prayer_times_from_api` function in `main.rs` to change the API URL.
- **Audio Files**: Place audio files for each prayer time in the `adhanapp/media/fajr` and `adhanapp/media/other` directories for Fajr and other prayers, respectively.

## Contributing
Contributions are welcome! Please fork the repository, make your changes, and submit a pull request.

## License
This project is licensed under the [MIT License](LICENSE).

