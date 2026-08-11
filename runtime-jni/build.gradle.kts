plugins {
    alias(libs.plugins.kotlin.jvm)
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    compilerOptions.jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
}

dependencies {
    api(project(":runtime-api"))
    // Track A L2 (Cpu path). Requires `./gradlew publishEngineeredToMavenLocal` in wasi-webgpu-jvm-mvp.
    api(libs.wasi.webgpu.host.api)
    api(libs.wasi.webgpu.abi.cm)
}
