
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
