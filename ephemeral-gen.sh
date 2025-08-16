#!/usr/bin/env bash
set -euo pipefail

# Check for required environment variables
if [[ -z "${AGE_KEY_PUBLIC:-}" ]]; then
    echo "Error: AGE_KEY_PUBLIC not set"
    exit 1
fi

# Install required tools
echo "Installing required tools..."
cargo install rsign2 --quiet
cargo install rage --quiet

# Generate ephemeral minisign keypair
echo "Generating ephemeral minisign keypair..."
rsign generate -f -W -p minisign.pub -s minisign.key

# Extract key info for logging (masked)
key_content=$(tail -1 minisign.key)
masked_key="${key_content:0:32}[REDACTED]"
echo "::add-mask::${masked_key}"

# Extract public key ID for logging
pub_key_id=$(head -1 minisign.pub | sed 's/.*: //')
echo "Generated ephemeral key with ID: ${pub_key_id}"

# Encrypt private key with AGE
echo "Encrypting private key with AGE..."
rage -e -r "${AGE_KEY_PUBLIC}" minisign.key > minisign.key.age

# Remove unencrypted private key
rm -f minisign.key

echo "Ephemeral keypair generated and encrypted successfully"
ls -la minisign.*