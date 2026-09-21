#!/usr/bin/env bash
# FeatherSurf Android Packaging Script
#
# Builds APK and AAB packages for Android.

set -euo pipefail

VERSION="${1:-0.0.1}"
OUTPUT_DIR="${2:-target/release/bundle}"

echo "FeatherSurf Android Packaging"
echo "Version: $VERSION"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build the Rust binaries for Android targets
echo "Building Rust binaries for Android..."

# Build for different architectures
TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
    "i686-linux-android"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cargo build --release --target "$target"
done

# ── Android Project Setup ─────────────────────────────────────────

echo "Setting up Android project..."
ANDROID_DIR="$OUTPUT_DIR/android"
mkdir -p "$ANDROID_DIR/app/src/main/jniLibs"

# Copy native libraries
for target in "${TARGETS[@]}"; do
    case "$target" in
        aarch64-linux-android) ABI="arm64-v8a" ;;
        armv7-linux-androideabi) ABI="armeabi-v7a" ;;
        x86_64-linux-android) ABI="x86_64" ;;
        i686-linux-android) ABI="x86" ;;
    esac
    
    JNI_DIR="$ANDROID_DIR/app/src/main/jniLibs/$ABI"
    mkdir -p "$JNI_DIR"
    cp "target/$target/release/libfeathersurf.so" "$JNI_DIR/"
done

# Create AndroidManifest.xml
cat > "$ANDROID_DIR/app/src/main/AndroidManifest.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="dev.feathersurf.browser">

    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"
        android:maxSdkVersion="28" />
    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE"
        android:maxSdkVersion="32" />

    <application
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:roundIcon="@mipmap/ic_launcher_round"
        android:supportsRtl="true"
        android:theme="@style/Theme.FeatherSurf"
        android:usesCleartextTraffic="false">
        
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTask"
            android:configChanges="orientation|screenSize|keyboardHidden"
            android:windowSoftInputMode="adjustResize">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
            <intent-filter>
                <action android:name="android.intent.action.VIEW" />
                <category android:name="android.intent.category.DEFAULT" />
                <category android:name="android.intent.category.BROWSABLE" />
                <data android:scheme="http" />
                <data android:scheme="https" />
            </intent-filter>
            <intent-filter>
                <action android:name="android.intent.action.VIEW" />
                <category android:name="android.intent.category.DEFAULT" />
                <category android:name="android.intent.category.BROWSABLE" />
                <data android:scheme="feathersurf" />
            </intent-filter>
        </activity>
    </application>
</manifest>
EOF

# Create build.gradle
cat > "$ANDROID_DIR/app/build.gradle" << EOF
plugins {
    id 'com.android.application'
    id 'org.jetbrains.kotlin.android'
}

android {
    namespace 'dev.feathersurf.browser'
    compileSdk 34

    defaultConfig {
        applicationId "dev.feathersurf.browser"
        minSdk 29
        targetSdk 34
        versionCode 1
        versionName "$VERSION"

        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86_64', 'x86'
        }
    }

    buildTypes {
        release {
            minifyEnabled true
            shrinkResources true
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }
    }

    compileOptions {
        sourceCompatibility JavaVersion.VERSION_1_8
        targetCompatibility JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = '1.8'
    }
}

dependencies {
    implementation 'androidx.core:core-ktx:1.12.0'
    implementation 'androidx.appcompat:appcompat:1.6.1'
    implementation 'com.google.android.material:material:1.11.0'
}
EOF

# Create proguard-rules.pro
cat > "$ANDROID_DIR/app/proguard-rules.pro" << EOF
# FeatherSurf ProGuard rules

# Keep Rust JNI methods
-keep class dev.feathersurf.browser.NativeLib {
    *;
}

# Keep WebView
-keep class android.webkit.** { *; }
EOF

echo "Android project created at $ANDROID_DIR"

# ── Build APK ──────────────────────────────────────────────────────

echo "Building APK..."
cd "$ANDROID_DIR"
./gradlew assembleRelease

# Copy APK
cp app/build/outputs/apk/release/app-release-unsigned.apk "$OUTPUT_DIR/feathersurf-$VERSION.apk"

echo "APK created: $OUTPUT_DIR/feathersurf-$VERSION.apk"

# ── Build AAB ──────────────────────────────────────────────────────

echo "Building AAB..."
./gradlew bundleRelease

# Copy AAB
cp app/build/outputs/bundle/release/app-release.aab "$OUTPUT_DIR/feathersurf-$VERSION.aab"

echo "AAB created: $OUTPUT_DIR/feathersurf-$VERSION.aab"

# ── Summary ────────────────────────────────────────────────────────

echo ""
echo "Android packaging complete!"
echo "Output: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"
