#!/bin/bash
# Generate changelog from conventional commits

set -euo pipefail

echo "📝 Generating changelog from conventional commits..."

# Install conventional-changelog-cli if not present
if ! command -v conventional-changelog &> /dev/null; then
    echo "Installing conventional-changelog-cli..."
    npm install -g conventional-changelog-cli
fi

# Generate changelog
conventional-changelog -p angular -i CHANGELOG.md -s -r 0

echo "✅ Changelog generated successfully!"
echo ""
echo "Review the generated CHANGELOG.md and commit it if it looks good."
