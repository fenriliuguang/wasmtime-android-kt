plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.mavenPublish)
}

extra["wasmtime.publishedArtifactId"] = "host-dawn"
extra["wasmtime.publishedName"] = "Wasmtime Android Dawn host"
extra["wasmtime.publishedDescription"] =
    "Dawn C (NativeGpu) default + androidx dawn-jni leftover. Prefer android-webgpu unless BYO runtime."
apply(from = rootProject.file("gradle/wasmtime-publish.gradle"))

android {
    namespace = "io.github.fenriliuguang.wasmtime.android.host.dawn"
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
            // Recipe output `native/third_party/dawn-c/out/<abi>/libwebgpu_dawn.so`
            // (gitignored). Empty dir is fine when Cloud has no NDK.
            jniLibs.directories.add("${rootProject.projectDir}/native/third_party/dawn-c/out")
        }
    }

    packaging {
        jniLibs {
            // Product default is one C API `.so`. androidx bundled is dawn-jni leftover.
            excludes += "**/libwebgpu_c_bundled.so"
        }
    }
}

kotlin {
    compilerOptions.jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
}

dependencies {
    api(project(":runtime-jni"))
    // dawn-jni leftover Kotlin (`DawnWasiWebGpuHost`). Bundled .so is packaging-excluded.
    api(libs.androidx.webgpu)
}
