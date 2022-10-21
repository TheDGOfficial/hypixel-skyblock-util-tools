#!/bin/bash
echo "Starting flush of target/ on RAM to disk"

# Copy target contents on RAM to disk
echo "Copying target/ into target-copy/"
cp -Lr target/ target-copy/

# Delete target contents on RAM
echo "Deleting contents of target/ on RAM and unmounting the ramdisk tmpfs"
cargo-ramdisk unmount

# Rename target contents on disk to its original name
echo "Renaming target-copy/ to target/"
mv target-copy/ target/

echo "Finished flush of target/ on RAM to disk"

