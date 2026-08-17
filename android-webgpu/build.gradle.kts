plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "io.github.fenriliuguang.wasmtime.android.webgpu.bundle"
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

dependencies {
    api(project(":android"))
    api(project(":host-dawn"))
}
