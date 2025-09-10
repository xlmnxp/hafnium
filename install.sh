#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="hf"
INSTALL_DIR="/usr/local/bin"
SERVICE_NAME="hafnium"
SERVICE_FILE="hafnium.service"
SYSTEMD_DIR="/etc/systemd/system"
HAFNIUM_USER="hafnium"
HAFNIUM_GROUP="hafnium"
HAFNIUM_HOME="/var/lib/hafnium"
HAFNIUM_RUN="/var/run"
HAFNIUM_LOG="/var/log"

echo -e "${GREEN}Installing Hafnium...${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}"
   exit 1
fi

# Build the project
echo -e "${YELLOW}Building Hafnium...${NC}"
cargo build --release

# Create hafnium user and group
echo -e "${YELLOW}Creating hafnium user and group...${NC}"
if ! id "$HAFNIUM_USER" &>/dev/null; then
    useradd --system --home-dir "$HAFNIUM_HOME" --shell /bin/false "$HAFNIUM_USER"
    echo -e "${GREEN}Created user: $HAFNIUM_USER${NC}"
else
    echo -e "${YELLOW}User $HAFNIUM_USER already exists${NC}"
fi

# Create directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p "$HAFNIUM_HOME"
mkdir -p "$HAFNIUM_RUN"
mkdir -p "$HAFNIUM_LOG"

# Set ownership
chown -R "$HAFNIUM_USER:$HAFNIUM_GROUP" "$HAFNIUM_HOME"
chown -R "$HAFNIUM_USER:$HAFNIUM_GROUP" "$HAFNIUM_RUN"
chown -R "$HAFNIUM_USER:$HAFNIUM_GROUP" "$HAFNIUM_LOG"

# Install binary
echo -e "${YELLOW}Installing binary...${NC}"
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
chown root:root "$INSTALL_DIR/$BINARY_NAME"

# Install systemd service
echo -e "${YELLOW}Installing systemd service...${NC}"
cp "$SERVICE_FILE" "$SYSTEMD_DIR/"
systemctl daemon-reload
systemctl enable "$SERVICE_NAME"

# Set up logrotate
echo -e "${YELLOW}Setting up log rotation...${NC}"
cat > /etc/logrotate.d/hafnium << EOF
/var/log/hafnium.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 644 $HAFNIUM_USER $HAFNIUM_GROUP
    postrotate
        systemctl reload $SERVICE_NAME > /dev/null 2>&1 || true
    endscript
}
EOF

# Create configuration file if it doesn't exist
if [[ ! -f "$HAFNIUM_HOME/hafnium.toml" ]]; then
    echo -e "${YELLOW}Creating initial configuration...${NC}"
    sudo -u "$HAFNIUM_USER" "$INSTALL_DIR/$BINARY_NAME" gen-key
fi

echo -e "${GREEN}Installation completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Configure your server: sudo -u $HAFNIUM_USER $INSTALL_DIR/$BINARY_NAME gen-key"
echo "2. Start the service: systemctl start $SERVICE_NAME"
echo "3. Check status: systemctl status $SERVICE_NAME"
echo "4. View logs: journalctl -u $SERVICE_NAME -f"
echo ""
echo -e "${GREEN}Service management commands:${NC}"
echo "  Start:   systemctl start $SERVICE_NAME"
echo "  Stop:    systemctl stop $SERVICE_NAME"
echo "  Restart: systemctl restart $SERVICE_NAME"
echo "  Status:  systemctl status $SERVICE_NAME"
echo "  Logs:    journalctl -u $SERVICE_NAME -f"