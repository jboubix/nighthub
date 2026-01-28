#!/usr/bin/env python3
"""
Test script to debug refresh indicators
"""

import subprocess
import time
import os
import signal


def test_refresh_debug():
    """Test refresh indicators with debugging"""

    # Set up test environment with real repos
    os.environ["GITHUB_TOKEN"] = "ghp_test1234567890abcdef1234567890abcdef12345678"
    os.environ["REPOS"] = "torvalds/linux,rust-lang/rust,microsoft/vscode"

    print("Testing refresh indicators with debugging...")
    print("Press 'f' after starting to trigger refresh")
    print("Check /tmp/refresh_debug.log for refresh state")
    print("Press Ctrl+C to stop")

    try:
        # Start nighthub
        process = subprocess.Popen(
            ["./target/debug/nighthub"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            preexec_fn=os.setsid,
        )

        # Monitor debug log
        last_log_content = ""
        try:
            while True:
                # Check debug log
                try:
                    with open("/tmp/refresh_debug.log", "r") as f:
                        log_content = f.read().strip()
                        if log_content != last_log_content:
                            print(f"DEBUG: {log_content}")
                            last_log_content = log_content
                except FileNotFoundError:
                    pass

                time.sleep(0.1)

        except KeyboardInterrupt:
            print("\nStopping test...")
            os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            process.wait()

    except Exception as e:
        print(f"Error: {e}")

    finally:
        # Clean up
        try:
            os.remove("/tmp/refresh_debug.log")
        except:
            pass
        os.environ.pop("GITHUB_TOKEN", None)
        os.environ.pop("REPOS", None)


if __name__ == "__main__":
    test_refresh_debug()
