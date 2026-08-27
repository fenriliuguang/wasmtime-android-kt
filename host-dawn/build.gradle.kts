plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.mavenPublish)
}

extra["wasmtime.publishedArtifactId"] = "host-dawn"
extra["wasmtime.publishedName"] = "Wasmtime Android Dawn host"
extra["wasmtime.publishedDescription"] =
    "Dawn/androidx.webgpu backend for wasmtime-android-kt. Prefer android-webgpu unless BYO runtime."
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
}

kotlin {
    compilerOptions.jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
}

dependencies {
    api(project(":runtime-jni"))
    // Dawn Java + bundled .so (not git). Bump via changelog when changing the pin.
    api(libs.androidx.webgpu)
}
