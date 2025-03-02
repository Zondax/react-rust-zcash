# Directories
JAVA_SRC_DIR = native_rust_lib/java/expo/modules/myrustmodule
TARGET_DIR = native_rust_lib/target/classes

CURRENT_DIR := $(shell pwd)
ABSOLUTE_TARGET_DIR := $(CURRENT_DIR)/$(TARGET_DIR)

# Java files to mock JVM in rust unit tests
JAVA_FILES = $(wildcard $(JAVA_SRC_DIR)/*.java)

# Create target directory if it doesn't exist
$(TARGET_DIR):
	mkdir -p $(TARGET_DIR)


# Compile Java files
.PHONY: compile-java
compile-java: | $(TARGET_DIR)
	@echo "Compiling Java files from $(JAVA_SRC_DIR)..."
	javac -d $(TARGET_DIR) $(JAVA_FILES)
	@echo "Java files compiled successfully to $(TARGET_DIR)"

# Run JNI test
rust-test-jni: compile-java
	@echo "Running JNI test..."
	@echo "Setting absolute CLASSPATH to: $(ABSOLUTE_TARGET_DIR)"
	cd native_rust_lib && \
	_JAVA_OPTIONS="-Djava.class.path=$(ABSOLUTE_TARGET_DIR) -verbose:class" \
	CLASSPATH="$(ABSOLUTE_TARGET_DIR)" \
	RUST_BACKTRACE=1 \
	cargo test -- --nocapture
	# cargo test test_get_init_tx_data -- --nocapture

# Rule to build the Android project
android-build:
	@echo "Building Android project..."
	npm run cargo-android
	npx expo prebuild --platform android

# Rule to run the Android project
android-run:
	@echo "Running Android project..."
	npm run android

# Default rule
all: android-build android-run

.PHONY: android-build android-run all
