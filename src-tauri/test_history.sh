#!/bin/bash
set -e

# Build
cargo build

# Clean up any existing test versions (optional, but good for local dev)
# rm -f ~/.hangar/versions/v*

# Create a dummy version (simulate AI or manual save)
# We can't easily trigger internal save_version from CLI without mocking or using "ai" command.
# But "ai" command requires an OpenAI key usually, or we can mock it?
# Actually "hangar history list" just reads files. We can manually create files to test listing and diffing.

VERSIONS_DIR="$HOME/.hangar/versions"
mkdir -p "$VERSIONS_DIR"

# Create v1
echo "version: 1" > "$VERSIONS_DIR/v1_basic_1000000001_test.yaml"
# Create v2
echo "version: 2" > "$VERSIONS_DIR/v2_basic_1000000002_test.yaml"

# List
echo "Listing versions:"
./target/debug/hangar history list

# Diff v1 v2
echo "Diffing v1 v2:"
./target/debug/hangar history diff v1 v2

# Diff v2 v1
echo "Diffing v2 v1:"
./target/debug/hangar history diff v2 v1

# Diff v1 v0 (should act as diff against empty)
echo "Diffing v1 v0:"
./target/debug/hangar history diff v1 v0

# Clean up
rm "$VERSIONS_DIR/v1_basic_1000000001_test.yaml"
rm "$VERSIONS_DIR/v2_basic_1000000002_test.yaml"
