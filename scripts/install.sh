#!/bin/bash

# Survon Council Seat Installer
# Installs the council seat binary and configures it

set -e

INSTALLER_VERSION="0.1.0"
BINARY_URL="https://github.com/survon/survon-council-seat/releases/latest/download/survon-council-seat"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/home/survon/data"

# Parse arguments
STRATEGY="librarian"
SKIP_INSTALL=0

while [[ $# -gt 0 ]]; do
    case $1 in
        --strategy)
            STRATEGY="$2"
            shift 2
            ;;
        --skip-install)
            SKIP_INSTALL=1
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "=========================================="
echo "Survon Council Seat Installer v${INSTALLER_VERSION}"
echo "=========================================="

# Check if running as non-root
if [ "$EUID" -eq 0 ]; then
    echo "Warning: Running as root. This installer is designed to run as the 'survon' user."
    read -p "Continue anyway? (y/n): " confirm
    if [ "$confirm" != "y" ]; then
        exit 0
    fi
fi

# Create data directory
echo "Creating data directory..."
mkdir -p "$DATA_DIR"

if [ $SKIP_INSTALL -eq 0 ]; then
    # Download binary
    echo "Downloading council seat binary..."
    curl -L "$BINARY_URL" -o /tmp/survon-council-seat
    
    if [ $? -ne 0 ]; then
        echo "Error: Failed to download binary"
        exit 1
    fi
    
    # Install binary
    echo "Installing binary..."
    sudo mv /tmp/survon-council-seat "$INSTALL_DIR/survon-council-seat"
    sudo chmod +x "$INSTALL_DIR/survon-council-seat"
    
    echo "Binary installed successfully"
fi

# Configure strategy
echo "Configuring strategy: $STRATEGY"

# Add to .bashrc for persistence
echo "export COUNCIL_STRATEGY=$STRATEGY" >> ~/.bashrc
echo "export DATABASE_PATH=$DATA_DIR/council.db" >> ~/.bashrc
echo "export LOG_LEVEL=info" >> ~/.bashrc

# Source the config
source ~/.bashrc

echo ""
echo "=========================================="
echo "Installation Complete!"
echo "=========================================="
echo "Strategy: $STRATEGY"
echo "Binary: $INSTALL_DIR/survon-council-seat"
echo "Data: $DATA_DIR"
echo ""
echo "To start the council seat:"
echo "  $INSTALL_DIR/survon-council-seat"
echo ""
echo "To change strategy later, edit ~/.bashrc or run:"
echo "  export COUNCIL_STRATEGY=<strategy>"
echo ""
echo "Available strategies: librarian, medicine, mechanical, botany, veterinary, building, survival"
echo ""
