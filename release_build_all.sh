#!/bin/bash
echo "Building for 64-bit Linux"
CROSS_COMPILE=true . ./release_build.sh --portable

# shellcheck disable=SC2317
echo "Building for 32-bit Linux"
# shellcheck disable=SC2317
CROSS_COMPILE=true . ./release_build.sh --portable --i686

# shellcheck disable=SC2317
echo "Building for aarch64 Linux"
# shellcheck disable=SC2317
CROSS_COMPILE=true . ./release_build.sh --portable --aarch64

# shellcheck disable=SC2317
echo "Building for 64-bit Windows"
# shellcheck disable=SC2317
CROSS_COMPILE=true . ./release_build.sh --portable --windows

# shellcheck disable=SC2317
echo "Building for 32-bit Windows"
# shellcheck disable=SC2317
CROSS_COMPILE=true . ./release_build.sh --portable --windows --i686

# shellcheck disable=SC2317
echo "Building for 64-bit macOS"
# shellcheck disable=SC2317
CROSS_COMPILE=true . ./release_build.sh --portable --macos

# shellcheck disable=SC2317
echo "Building for aarch64 macOS"
# shellcheck disable=SC2317
CROSS_COMPILE=true . ./release_build.sh --portable --macos --aarch64
