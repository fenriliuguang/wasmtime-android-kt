plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.mavenPublish)
}

extra["wasmtime.publishedArtifactId"] = "runtime"
extra["wasmtime.publishedName"] = "Wasmtime Android runtime"
extra["wasmtime.publishedDescription"] =
    "Android AAR with Wasmtime JNI, Component Model SPI, and libwasmtime_android_kt.so (no Dawn)."
apply(from = rootProject.file("gradle/wasmtime-publish.gradle"))

android {
    namespace = "io.github.fenriliuguang.wasmtime.android"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    sourceSets {
        getByName("main") {
            // Produced by scripts/build-native-android.ps1
            jniLibs.directories.add("jniLibs")
        }
    }

    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
}

dependencies {
    api(project(":runtime-jni"))
}
