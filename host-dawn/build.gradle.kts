plugins {
    alias(libs.plugins.android.library)
}

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
    // Unpublished until vendored or published — mavenLocal only for this module.
    api(libs.wasi.webgpu.host.api)
    api(libs.wasi.webgpu.host.webgpu)
    implementation(libs.wasi.webgpu.abi.cm)
}
