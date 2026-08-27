plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.mavenPublish)
}

extra["wasmtime.publishedArtifactId"] = "android-webgpu"
extra["wasmtime.publishedName"] = "Wasmtime Android WebGPU bundle"
extra["wasmtime.publishedDescription"] =
    "Default 0.x product bundle: runtime + Dawn host. Recommended consumer coordinate."
apply(from = rootProject.file("gradle/wasmtime-publish.gradle"))

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
