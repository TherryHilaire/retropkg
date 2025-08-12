#!/bin/bash
set -e

# Create test package
mkdir -p test-pkg/data/usr/bin
echo 'echo "Hello RetroPKG!"' > test-pkg/data/usr/bin/retro-test
chmod +x test-pkg/data/usr/bin/retro-test

cat > test-pkg/manifest.toml <<'EOL'
[package]
name = "retro-test"
version = "1.0"
arch = "aarch64"
description = "Test package"
retroscore = 1
EOL

# Build package
tar czf retro-test.retro -C test-pkg manifest.toml data

# Build package manager
cargo build --release

# Create database directory
sudo mkdir -p /var/lib/retropkg
sudo chmod 777 /var/lib/retropkg

# Install package
sudo ./target/release/retropkg install --file retro-test.retro

# List packages
./target/release/retropkg list

# Run test
/usr/bin/retro-test

# Remove package
sudo ./target/release/retropkg remove --name retro-test
