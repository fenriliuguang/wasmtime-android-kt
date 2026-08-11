plugins {
    alias(libs.plugins.android.application)
}

val copyM1Fixtures by tasks.registering(Copy::class) {
    from(rootProject.file("fixtures/m1")) {
        include("*.wasm")
    }
    into(layout.projectDirectory.dir("src/androidTest/assets/m1"))
}

tasks.named("preBuild").configure { dependsOn(copyM1Fixtures) }

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
        versionName = "0.1.0-experimental"

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
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
    // Lets AGP/UTP install androidx.test.services (needed on API 30+ / OEM devices).
    androidTestUtil(libs.androidx.test.services)
}
