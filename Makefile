.PHONY: build install uninstall clean test

# Build the project
build:
	cargo build --release

# Install the service
install: build
	sudo ./install.sh

# Uninstall the service
uninstall:
	sudo systemctl stop hafnium || true
	sudo systemctl disable hafnium || true
	sudo rm -f /etc/systemd/system/hafnium.service
	sudo rm -f /usr/local/bin/hf
	sudo rm -f /etc/logrotate.d/hafnium
	sudo userdel hafnium || true
	sudo rm -rf /var/lib/hafnium
	sudo systemctl daemon-reload

# Clean build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo test

# Start the service
start:
	sudo systemctl start hafnium

# Stop the service
stop:
	sudo systemctl stop hafnium

# Restart the service
restart:
	sudo systemctl restart hafnium

# Show service status
status:
	sudo systemctl status hafnium

# Show service logs
logs:
	sudo journalctl -u hafnium -f

# Generate keys
genkey:
	sudo -u hafnium /usr/local/bin/hf gen-key

# Show public key
pubkey:
	sudo -u hafnium /usr/local/bin/hf get-self