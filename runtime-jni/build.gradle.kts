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
    testImplementation(libs.junit)
}

// Optional desktop shell: load host cdylib from desktop/jniLibs (see docs/contribute.md).
val desktopJniLibs = rootProject.layout.projectDirectory.dir("desktop/jniLibs")
tasks.test {
    val jniPath = desktopJniLibs.asFile.absolutePath
    systemProperty("java.library.path", jniPath)
    systemProperty("wasmtime.desktop.jniLibs", jniPath)
    val sep = System.getProperty("path.separator")
    val pathKey = if (System.getProperty("os.name").orEmpty().startsWith("Windows")) "Path" else "PATH"
    environment(pathKey, jniPath + sep + (System.getenv(pathKey) ?: System.getenv("PATH") ?: ""))
}
