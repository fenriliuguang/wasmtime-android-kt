import org.gradle.api.publish.PublishingExtension
import org.gradle.api.publish.maven.MavenPublication
import org.gradle.api.credentials.PasswordCredentials

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.mavenPublish) apply false
}

val publishedGroup = providers.gradleProperty("wasmtime.android.group")
val publishedVersion = providers.gradleProperty("wasmtime.android.version")

subprojects {
    group = publishedGroup.get()
    version = publishedVersion.get()

    pluginManager.withPlugin("maven-publish") {
        extensions.configure<PublishingExtension>("publishing") {
            repositories {
                maven {
                    name = "githubPackages"
                    url = uri("https://maven.pkg.github.com/fenriliuguang/wasmtime-android-kt")
                    credentials(PasswordCredentials::class)
                }
            }
        }
        // :android publishes as artifactId `runtime`. Rewrite sibling POMs.
        afterEvaluate {
            extensions.findByType<PublishingExtension>()
                ?.publications
                ?.withType<MavenPublication>()
                ?.configureEach {
                    pom.withXml {
                        val deps = asNode().children().filterIsInstance<groovy.util.Node>().find { node ->
                            node.name().toString().contains("dependencies")
                        } ?: return@withXml
                        deps.children().filterIsInstance<groovy.util.Node>().forEach { dep ->
                            val artifact = dep.children().filterIsInstance<groovy.util.Node>().find { node ->
                                node.name().toString().contains("artifactId")
                            } ?: return@forEach
                            if (artifact.text() == "android") {
                                artifact.setValue("runtime")
                            }
                        }
                    }
                }
        }
    }
}
