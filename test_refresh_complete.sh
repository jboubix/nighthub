#!/bin/bash

echo "=== Nighthub Refresh Indicator Test ==="
echo ""
echo "This test will verify that refresh indicators work correctly."
echo ""

# Set up environment
export GITHUB_TOKEN="ghp_test1234567890abcdef1234567890abcdef12345678"
export REPOS="torvalds/linux,rust-lang/rust"

echo "Environment set up."
echo "GITHUB_TOKEN: ${GITHUB_TOKEN:0:20}..."
echo "REPOS: $REPOS"
echo ""

# Build the application
echo "Building nighthub..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Test the refresh logic directly
echo "Testing refresh logic..."
cargo run --bin test_refresh_simple
echo ""

echo "=== Instructions for Manual Testing ==="
echo "1. Run: ./target/release/nighthub"
echo "2. Press 'f' to trigger refresh"
echo "3. Look for:"
echo "   - Header: '🔄 ⠋ Refreshing 2 repos...'"
echo "   - Repository lines: '🔄 torvalds/linux: 0' and '🔄 rust-lang/rust: 0'"
echo "4. The refresh should take about 1 second due to mock delays"
echo "5. Press 'q' to quit"
echo ""

echo "=== Expected Behavior ==="
echo "✅ When NOT refreshing:"
echo "   Header: 'Refresh in Xs'"
echo "   Repos: 'torvalds/linux: 0', 'rust-lang/rust: 0'"
echo ""
echo "✅ When refreshing (after pressing 'f'):"
echo "   Header: '🔄 ⠋ Refreshing 2 repos...'"
echo "   Repos: '🔄 torvalds/linux: 0', '🔄 rust-lang/rust: 0'"
echo ""

echo "=== Troubleshooting ==="
echo "If you don't see the emojis:"
echo "1. Check if your terminal supports emoji (try: echo '🔄 test')"
echo "2. Try running in a different terminal (gnome-terminal, iTerm2, etc.)"
echo "3. Make sure you're pressing 'f' when no popup is open"
echo "4. The refresh state only lasts ~1 second with mock data"
echo ""

read -p "Press Enter to run nighthub for manual testing..."

echo "Starting nighthub..."
echo "(Press 'q' to quit, 'f' to refresh)"
echo ""

./target/release/nighthub