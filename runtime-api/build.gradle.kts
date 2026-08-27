plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.mavenPublish)
}

extra["wasmtime.publishedArtifactId"] = "runtime-api"
extra["wasmtime.publishedName"] = "Wasmtime Android runtime API"
extra["wasmtime.publishedDescription"] =
    "Public Kotlin SPI for wasmtime-android-kt. Transitive of runtime; do not depend on this GAV directly."
apply(from = rootProject.file("gradle/wasmtime-publish.gradle"))

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    compilerOptions.jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
}
