#!/usr/bin/env bash
set -euo pipefail

# Validate environment
if [[ -z "${AGE_KEY_SECRET:-}" ]]; then
    echo "Error: AGE_KEY_SECRET not set"
    exit 1
fi

if [[ -z "${GITHUB_REPOSITORY:-}" ]]; then
    echo "Warning: GITHUB_REPOSITORY not set, using 'unknown'"
    GITHUB_REPOSITORY="unknown"
fi

if [[ -z "${GITHUB_RUN_ID:-}" ]]; then
    echo "Warning: GITHUB_RUN_ID not set, using 'unknown'"
    GITHUB_RUN_ID="unknown"
fi

# Create temporary AGE key file
age_key_file=$(mktemp age.key.XXXXXXXXXX)
trap 'rm -f "${age_key_file}" minisign.key' EXIT

# Write AGE secret to temp file
echo "${AGE_KEY_SECRET}" > "${age_key_file}"

# Decrypt minisign private key
echo "Decrypting minisign private key..."
rage -d -i "${age_key_file}" minisign.key.age > minisign.key

# Get metadata for trusted comment
timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")
git_commit=$(git rev-parse HEAD 2>/dev/null || echo "unknown")

# Create trusted comment with metadata
comment="gh=${GITHUB_REPOSITORY} git=${git_commit} ts=${timestamp} run=${GITHUB_RUN_ID}"

echo "Signing with metadata: ${comment}"

# Sign each file passed as argument
for file in "$@"; do
    if [[ -f "$file" ]]; then
        echo "Signing ${file}..."
        rsign sign -W -s minisign.key -t "${comment}" "${file}"
        
        # Rename .minisig to .sig for consistency with cargo-binstall
        if [[ -f "${file}.minisig" ]]; then
            mv "${file}.minisig" "${file}.sig"
        fi
        
        echo "Created signature: ${file}.sig"
    else
        echo "Warning: File not found: ${file}"
    fi
done

echo "Signing complete"