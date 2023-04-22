#!/bin/bash
echo "Building for 64-bit Linux"
CROSS_COMPILE=true . ./release_build.sh --portable

echo "Building for 32-bit Linux"
CROSS_COMPILE=true . ./release_build.sh --portable --i686

echo "Building for aarch64 Linux"
CROSS_COMPILE=true . ./release_build.sh --portable --aarch64

echo "Building for 64-bit Windows"
CROSS_COMPILE=true . ./release_build.sh --portable --windows

echo "Building for 32-bit Windows"
CROSS_COMPILE=true . ./release_build.sh --portable --windows --i686

echo "Building for 64-bit macOS"
CROSS_COMPILE=true . ./release_build.sh --portable --macos

echo "Building for aarch64 macOS"
CROSS_COMPILE=true . ./release_build.sh --portable --macos --aarch64
