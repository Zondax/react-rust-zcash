#!/bin/bash

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Enable X11 access for Docker
xhost +local:docker

# Get host IP for Android
export ANDROID_HOST_IP=$(ip route get 8.8.8.8 | awk '{print $7}')

# Remove existing container if it exists
docker rm -f expo-rust-dev 2>/dev/null || true

# Run the container in interactive mode
docker run -it \
    --privileged \
    --network host \
    -e ANDROID_HOST_IP=$ANDROID_HOST_IP \
    -e DISPLAY=$DISPLAY \
    -e ANDROID_HOME=/opt/android-sdk \
    -e ANDROID_NDK_HOME=/opt/android-sdk/ndk/25.2.9519653 \
    -v "${PROJECT_ROOT}":/app \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    -v $HOME/.android:/root/.android \
    -v $HOME/Android/Sdk/platform-tools:/opt/android-sdk/platform-tools \
    -v $HOME/Android/Sdk/emulator:/opt/android-sdk/emulator \
    --device /dev/kvm \
    --device /dev/bus/usb \
    --name expo-rust-dev \
    expo-rust-android \
    /bin/bash
