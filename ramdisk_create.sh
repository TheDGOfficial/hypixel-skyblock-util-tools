#!/bin/bash

# Will install cargo-ramdisk if not already installed
echo "Checking install of cargo-ramdisk"
cargo install cargo-ramdisk

echo "Starting creation of ramdisk for target/"

# Create target folder if it doesn't exist to prevent any errors in later commands
# Ideally we should check if the folder exists before doing any operation, but this an easy fix without any big effects.
echo "Creating target/ directory if it doesn't exist"
mkdir -p target/

# Copy target contents on disk to another folder on disk
echo "Copying target/ into target-copy/"
cp -Lr target/ target-copy/

# Mount a ram disk at target/, deleting its contents, hence the copy above.
echo "Deleting contents of target/ on disk and mounting the ramdisk tmpfs"
cargo-ramdisk

# Now copy contents from our copy after mounting ram disk, to RAM.
echo "Copying contents of target-copy/ in disk into target/ on RAM"
cp -Lr target-copy/. target/

# We no longer need this folder, unless a power failure occurs, the code at the end will flush the contents of target on ram to the disk. And if a power failure occurs, the build cache on target/ would be most probably corrupted anyways, so no need to bother handling it.
echo "Deleting target-copy/ in disk since it's no longer needed"
rm -r target-copy/

echo "Finished creation of ramdisk for target/"
