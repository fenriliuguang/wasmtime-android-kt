plugins {
    alias(libs.plugins.android.application)
}

val copyM1Fixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/m1")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/m1"))
}

val copyM2Fixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/m2")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/m2"))
}

val copyM3Fixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/m3")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/m3"))
}

val copyM4Fixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/m4")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/m4"))
}

val copyP3Fixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/p3")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/p3"))
}

val copyWasiFixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/wasi")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/wasi"))
}

tasks.named("preBuild").configure {
    dependsOn(
        copyM1Fixtures,
        copyM2Fixtures,
        copyM3Fixtures,
        copyM4Fixtures,
        copyP3Fixtures,
        copyWasiFixtures,
    )
}

android {
    namespace = "io.github.fenriliuguang.wasmtime.android.smoke"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        applicationId = "io.github.fenriliuguang.wasmtime.android.smoke"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = providers.gradleProperty("wasmtime.android.version").get()

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildTypes {
        release {
            optimization {
                enable = false
            }
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
}

dependencies {
    implementation(project(":android"))
    implementation(libs.androidx.core.ktx)
    // Dawn native (`libwebgpu_c_bundled.so`) must live in the main APK (Track A pattern).
    implementation(libs.wasi.webgpu.host.webgpu)
    // Track A L2 Cpu path for M3 instruments (also via :runtime-jni).
    androidTestImplementation(libs.wasi.webgpu.host.api)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
    // Lets AGP/UTP install androidx.test.services (needed on API 30+ / OEM devices).
    androidTestUtil(libs.androidx.test.services)
}
