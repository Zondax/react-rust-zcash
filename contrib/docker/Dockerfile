# Use Ubuntu as base image
FROM ubuntu:22.04
# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
# Set environment variables
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/25.2.9519653
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_NDK_HOME}:${JAVA_HOME}/bin
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
    openjdk-17-jre \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Verify Java installation
# Install Java and verify installation
RUN apt-get update && apt-get install -y \
    openjdk-17-jdk \
    openjdk-17-jre \
    && rm -rf /var/lib/apt/lists/* \
    && update-alternatives --config java

ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
ENV PATH=$PATH:$JAVA_HOME/bin

# Verify Java installation
RUN which java && \
    java -version && \
    echo "JAVA_HOME=$JAVA_HOME" && \
    ls -la $JAVA_HOME/bin/java

# Install Node.js (rest of your Node.js installation remains the same)
RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Rust installation remains the same
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $CARGO_HOME/env \
    && rustup target add \
        aarch64-linux-android \
        armv7-linux-androideabi \
        i686-linux-android \
        x86_64-linux-android

# Install cargo-ndk
RUN cargo install cargo-ndk

# Install Android SDK and NDK with updated versions
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools/latest && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip && \
    unzip -q commandlinetools-linux-9477386_latest.zip && \
    mv cmdline-tools/* ${ANDROID_HOME}/cmdline-tools/latest/ && \
    rm -rf cmdline-tools commandlinetools-linux-9477386_latest.zip

# Updated SDK components installation
RUN mkdir -p $ANDROID_HOME && \
    yes | sdkmanager --licenses && \
    sdkmanager --verbose \
        "platform-tools" \
        "platforms;android-34" \
        "build-tools;34.0.0" \
        "ndk;25.2.9519653" \
        "cmake;3.22.1" && \
    sdkmanager --update && \
    # Verify NDK installation
    echo "Verifying NDK installation..." && \
    sdkmanager --list_installed | grep ndk

# Make directories and set permissions
RUN mkdir -p ${ANDROID_HOME}/ndk && \
    chmod -R 755 ${ANDROID_HOME} && \
    mkdir -p /root/.gradle && \
    chmod -R 777 /root/.gradle

# Install additional Expo tools
RUN npm install -g expo-cli @expo/cli

# Rest remains the same
WORKDIR /app
COPY package*.json ./
COPY app.json ./
RUN npm install
COPY . .
EXPOSE 8081 19000 19001 5037

# Modified command to ensure proper initialization
CMD ["/bin/bash", "-c", "npm run cargo-android && npx expo prebuild --platform android && npm run android"]
