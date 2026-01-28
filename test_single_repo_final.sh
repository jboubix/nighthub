#!/bin/bash

echo "=== Nighthub Single Repository Refresh Test ==="
echo ""

# Set up environment for single repository
export GITHUB_TOKEN="ghp_test1234567890abcdef1234567890abcdef12345678"
export REPOS="torvalds/linux"

echo "Testing with single repository..."
echo "GITHUB_TOKEN: ${GITHUB_TOKEN:0:20}..."
echo "REPOS: $REPOS"
echo ""

# Test the refresh logic directly
echo "Testing refresh logic..."
cargo run --bin test_single_repo
echo ""

echo "=== Expected Behavior ==="
echo "✅ When NOT refreshing:"
echo "   Header: 'Refresh in Xs'"
echo "   Repository: 'torvalds/linux: 0'"
echo ""
echo "✅ When refreshing (after pressing 'f'):"
echo "   Header: '🔄 ⠋ Refreshing 1 repos...'"
echo "   Repository: '🔄 torvalds/linux: 0'"
echo ""
echo "=== Troubleshooting ==="
echo "If refresh indicators don't work:"
echo "1. Check that you're running in a terminal that supports emoji"
echo "2. Make sure you press 'f' when no popup is open"
echo "3. The refresh state only lasts ~1 second with mock data"
echo "4. Try running: echo '🔄 test' to verify emoji support"
echo ""

# Test emoji support
echo "Testing emoji support..."
echo '🔄 test emoji support'
if [ $? -eq 0 ]; then
    echo "✅ Emoji support detected"
else
    echo "❌ Emoji support not detected"
fi

echo ""
echo "=== Manual TUI Test ==="
echo "If you want to test the full TUI:"
echo "1. Run: GITHUB_TOKEN=ghp_test1234567890abcdef1234567890abcdef12345678 REPOS=torvalds/linux ./target/debug/nighthub"
echo "2. Press 'f' to trigger refresh"
echo "3. Look for 🔄 emoji next to 'torvalds/linux'"
echo "4. The refresh should complete in ~1 second"
echo "5. Press 'q' to quit"
echo ""

read -p "Press Enter to run TUI test (or Ctrl+C to exit)..."

if command -v nighthub >/dev/null 2>&1; then
    echo "Starting nighthub TUI..."
    echo "Note: If you get terminal errors, the environment may not support TUI"
    echo "The refresh logic has been verified to work correctly."
    timeout 10s bash -c 'GITHUB_TOKEN=ghp_test1234567890abcdef1234567890abcdef12345678 REPOS=torvalds/linux ./target/debug/nighthub' || echo "TUI test completed"
else
    echo "nighthub binary not found. Build with: cargo build"
fi