# Update package list and install dependencies
sudo apt-get update 
sudo apt-get install -y make git build-essential pkg-config libusb-1.0-0-dev libsdl2-dev bsdmainutils

# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the Rust environment
source $HOME/.cargo/env

# Run make
make
