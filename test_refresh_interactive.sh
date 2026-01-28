#!/bin/bash

echo "Testing nighthub refresh indicators..."
echo "Setting up environment..."

export GITHUB_TOKEN="ghp_test1234567890abcdef1234567890abcdef12345678"
export REPOS="torvalds/linux,rust-lang/rust"

echo "Starting nighthub..."
echo "Instructions:"
echo "1. Wait for nighthub to start"
echo "2. Press 'f' to trigger refresh"
echo "3. Look for 🔄 emojis next to repository names"
echo "4. Look for '🔄 Refreshing X repos...' in header"
echo "5. Press 'q' to quit"
echo ""

# Run nighthub with a script that will show expected behavior
./target/debug/nighthub &
NIGHTUB_PID=$!

# Give it time to start
sleep 2

echo ""
echo "Nighthub should now be running."
echo "If you don't see the UI, the terminal might not support it."
echo ""
echo "Expected behavior when you press 'f':"
echo "- Header should show: '🔄 ⠋ Refreshing 2 repos...'"
echo "- Repository lines should show: '🔄 torvalds/linux: 0' and '🔄 rust-lang/rust: 0'"
echo ""

# Wait for user to stop
echo "Press Enter to stop nighthub..."
read

# Clean up
kill $NIGHTUB_PID 2>/dev/null
echo "Test completed."