#!/bin/bash
# Script to scan and export KCura CLI help text to documentation
# This ensures CLI documentation stays in sync with actual CLI implementation

set -e

echo "📋 Generating KCura CLI Help Documentation..."
echo "=============================================="

# Create docs directory if it doesn't exist
mkdir -p docs

# Generate main help
echo "# KCura CLI Reference" > docs/CLI_HELP.md
echo "" >> docs/CLI_HELP.md
echo "## Main Commands" >> docs/CLI_HELP.md
echo "" >> docs/CLI_HELP.md

# Build and capture main help
cargo build --release --package kcura-cli 2>/dev/null || cargo build --package kcura-cli
echo '```bash' >> docs/CLI_HELP.md
./target/release/kcura --help >> docs/CLI_HELP.md 2>&1 || ./target/debug/kcura --help >> docs/CLI_HELP.md 2>&1
echo '```' >> docs/CLI_HELP.md
echo "" >> docs/CLI_HELP.md

# Generate help for each subcommand
SUBCOMMANDS=("init" "convert" "query" "validate" "load" "register-hook" "on-commit" "receipt" "version")

for cmd in "${SUBCOMMANDS[@]}"; do
    echo "## \`kcura $cmd\` Command" >> docs/CLI_HELP.md
    echo "" >> docs/CLI_HELP.md
    echo '```bash' >> docs/CLI_HELP.md
    ./target/release/kcura "$cmd" --help >> docs/CLI_HELP.md 2>&1 || ./target/debug/kcura "$cmd" --help >> docs/CLI_HELP.md 2>&1 || echo "Command '$cmd' not available or failed"
    echo '```' >> docs/CLI_HELP.md
    echo "" >> docs/CLI_HELP.md
done

echo "✅ CLI help documentation generated in docs/CLI_HELP.md"

# Also generate a quick reference for README
echo "" >> docs/CLI_HELP.md
echo "## Quick Reference" >> docs/CLI_HELP.md
echo "" >> docs/CLI_HELP.md
echo "| Command | Description |" >> docs/CLI_HELP.md
echo "|---------|-------------|" >> docs/CLI_HELP.md

# Extract command descriptions (simplified)
echo "| \`kcura init\` | Initialize a new knowledge base |" >> docs/CLI_HELP.md
echo "| \`kcura convert\` | Convert OWL/SHACL to database schema |" >> docs/CLI_HELP.md
echo "| \`kcura query\` | Execute SPARQL queries |" >> docs/CLI_HELP.md
echo "| \`kcura validate\` | Validate SHACL constraints |" >> docs/CLI_HELP.md
echo "| \`kcura load\` | Load data into knowledge base |" >> docs/CLI_HELP.md
echo "| \`kcura register-hook\` | Register governance hooks |" >> docs/CLI_HELP.md
echo "| \`kcura on-commit\` | Execute on-commit hooks |" >> docs/CLI_HELP.md

echo "" >> docs/CLI_HELP.md
echo "*This documentation is auto-generated from the actual CLI implementation.*" >> docs/CLI_HELP.md
echo "*Last updated: $(date)*" >> docs/CLI_HELP.md
