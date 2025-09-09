#!/bin/sh

# Get the directory of the script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Define the path to the binary
BINARY_PATH="$SCRIPT_DIR/../target/debug/ai_messenger"

# Check if the binary exists
if [ ! -x "$BINARY_PATH" ]; then
  echo "Error: Binary not found or not executable at $BINARY_PATH"
  exit 1
fi

# Run the binary with the data command and remove the data dir
rm -rf $("$BINARY_PATH" data "$@")
