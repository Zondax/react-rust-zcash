#!/bin/bash
# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Check if emulator is already running
EMULATOR_RUNNING=$($HOME/Android/Sdk/platform-tools/adb devices | grep emulator | wc -l)

if [ "$EMULATOR_RUNNING" -eq 0 ]; then
    AVAILABLE_AVD=$($HOME/Android/Sdk/emulator/emulator -list-avds | head -n 1)
    if [ -z "$AVAILABLE_AVD" ]; then
        echo "Error: No Android Virtual Devices (AVDs) found!"
        exit 1
    fi
    echo "Starting Android emulator with AVD: $AVAILABLE_AVD"
    $HOME/Android/Sdk/emulator/emulator -avd $AVAILABLE_AVD &
    echo "Waiting for emulator to start..."
    $HOME/Android/Sdk/platform-tools/adb wait-for-device
else
    echo "Emulator already running, continuing..."
fi

# Remove existing container if it exists
docker rm -f expo-rust-dev 2>/dev/null || true

# Build the Docker image
docker build -t expo-rust-android -f "${PROJECT_ROOT}/contrib/docker/Dockerfile" "${PROJECT_ROOT}"

# Run the container
docker run -it \
    --privileged \
    --network host \
    -e ANDROID_HOST_IP=$(ip route get 8.8.8.8 | awk '{print $7}') \
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
    /bin/bash -c 'cd /app && npm run cargo-android && npx expo prebuild --platform android && npm run android'

# Clean up when done
echo "Cleaning up..."
$HOME/Android/Sdk/platform-tools/adb kill-server
