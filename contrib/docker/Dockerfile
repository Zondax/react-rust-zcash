# Use Ubuntu as base image
FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Set environment variables
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/25.2.9519653
ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_NDK_HOME}
ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup
ENV PATH=${PATH}:${CARGO_HOME}/bin

# Install basic dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    unzip \
    git \
    python3 \
    python3-pip \
    openjdk-17-jdk \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and required targets
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $CARGO_HOME/env \
    && rustup target add \
        aarch64-linux-android \
        armv7-linux-androideabi \
        i686-linux-android \
        x86_64-linux-android

# Install cargo-ndk
RUN cargo install cargo-ndk

# Install Android SDK and NDK
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools/latest && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip && \
    unzip -q commandlinetools-linux-9477386_latest.zip && \
    mv cmdline-tools/* ${ANDROID_HOME}/cmdline-tools/latest/ && \
    rm -rf cmdline-tools commandlinetools-linux-9477386_latest.zip

# Accept licenses and install required components
# Accept licenses and install required components with verification
RUN mkdir -p $ANDROID_HOME && \
    yes | sdkmanager --licenses && \
    sdkmanager --verbose \
        "platform-tools" \
        "platforms;android-31" \
        "build-tools;33.0.0" \
        "ndk;25.2.9519653" && \
    # Wait for NDK installation and verify
    echo "Verifying NDK installation..." && \
    sdkmanager --list_installed | grep ndk && \
    ls -la $ANDROID_HOME/ndk/ && \
    echo "ANDROID_NDK_HOME=$ANDROID_NDK_HOME" && \
    echo "NDK installation complete"

# Make sure the NDK directory is accessible
RUN chmod -R 755 $ANDROID_HOME

# Install Expo CLI
RUN npm install -g expo-cli

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./
COPY app.json ./

# Install project dependencies
RUN npm install

# Copy the rest of the project
COPY . .

# Expose ports
EXPOSE 8081 19000 19001 5037

# Command to build rust and start app
CMD ["/bin/bash", "-c", "npm run cargo-android && npm run android"]
