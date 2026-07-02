// Top-level build file where you can add configuration options common to all sub-projects/modules.
buildscript {
    repositories {
        google()
        maven { url = uri("https://developer.huawei.com/repo/") }
        maven {
            url = uri("https://jitpack.io")
            content {
                includeGroup("com.github.aasitnikov")
            }
        }
    }
    dependencies {
        classpath("com.android.tools.build:gradle:9.1.1")
        classpath("com.github.aasitnikov:fat-aar-android:1.4.6")
    }
}

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.compose) apply false
}

val pushHubGroupId = "com.jiangker.push"
val pushHubVersion: String =
    (findProperty("pushHubVersion") as? String)?.trim()?.takeIf { it.isNotEmpty() }
        ?: System.getenv("PUSH_HUB_VERSION")?.trim()?.takeIf { it.isNotEmpty() }
        ?: rootProject.file("VERSION").takeIf { it.exists() }?.readText()?.trim()?.ifEmpty { null }
        ?: "0.0.1-SNAPSHOT"

extra["pushHubGroupId"] = pushHubGroupId
extra["pushHubVersion"] = pushHubVersion

val publishableModules = listOf(
    "push-core",
    "push",
    "push-xiaomi",
    "push-huawei",
    "push-oppo",
    "push-vivo",
    "push-honor",
    "push-meizu",
)


subprojects {
    if (name !in publishableModules) return@subprojects

    group = pushHubGroupId
    version = pushHubVersion

    pluginManager.withPlugin("com.android.library") {
        apply(plugin = "maven-publish")

        extensions.configure<com.android.build.api.dsl.LibraryExtension>("android") {
            publishing {
                singleVariant("release") {
                    withSourcesJar()
                }
            }
        }

        afterEvaluate {
            extensions.configure<PublishingExtension>("publishing") {
                publications {
                    create<MavenPublication>("release") {
                        this.groupId = pushHubGroupId
                        this.artifactId = project.name
                        this.version = pushHubVersion
                        from(components["release"])
                        pom {
                            name.set("Push Hub ${project.name}")
                            description.set("Push Hub Android SDK module: ${project.name}")
                            url.set("https://github.com/jiangkerLove/push-hub")
                            licenses {
                                license {
                                    name.set("GPL-3.0")
                                    url.set("https://www.gnu.org/licenses/gpl-3.0.html")
                                }
                            }
                            developers {
                                developer {
                                    id.set("jiangkerLove")
                                    name.set("jiangkerLove")
                                }
                            }
                            scm {
                                url.set("https://github.com/jiangkerLove/push-hub")
                                connection.set("scm:git:git://github.com/jiangkerLove/push-hub.git")
                                developerConnection.set("scm:git:ssh://github.com/jiangkerLove/push-hub.git")
                            }
                        }
                    }
                }

                repositories {
                    mavenLocal()

                    val ghOwner = System.getenv("GITHUB_REPOSITORY_OWNER")
                        ?: (findProperty("gpr.owner") as? String)
                        ?: "jiangkerLove"
                    val ghRepo = System.getenv("GITHUB_REPOSITORY")
                        ?.substringAfter('/')
                        ?: (findProperty("gpr.repo") as? String)
                        ?: "push-hub"
                    val gprUser = System.getenv("GITHUB_ACTOR")
                        ?: (findProperty("gpr.user") as? String)
                    val gprKey = System.getenv("GITHUB_TOKEN")
                        ?: (findProperty("gpr.key") as? String)

                    if (!gprUser.isNullOrBlank() && !gprKey.isNullOrBlank()) {
                        maven {
                            name = "GitHubPackages"
                            url = uri("https://maven.pkg.github.com/$ghOwner/$ghRepo")
                            credentials {
                                username = gprUser
                                password = gprKey
                            }
                        }
                    }
                }
            }
        }
    }
}

tasks.register("assemblePushHubAars") {
    group = "build"
    description = "Assemble release AARs for all Push Hub modules"
    dependsOn(publishableModules.map { ":$it:assembleRelease" })
}

tasks.register("syncPushHubAarsToSample") {
    group = "build"
    description = "Build release AARs and copy them to sample/libs/ (versioned filenames)"
    dependsOn("assemblePushHubAars")
    doLast {
        val dest = rootDir.resolve("sample/libs")
        dest.mkdirs()
        publishableModules.forEach { module ->
            val src = rootDir.resolve("$module/build/outputs/aar/$module-release.aar")
            check(src.exists()) {
                "Missing $src — run assemblePushHubAars first"
            }
            val out = dest.resolve("$module-$pushHubVersion.aar")
            src.copyTo(out, overwrite = true)
            logger.lifecycle("Copied ${out.name}")
        }
    }
}

tasks.register("publishReleaseToGitHubPackages") {
    group = "publishing"
    description = "Publish Push Hub modules to GitHub Packages"
    dependsOn(publishableModules.map { ":$it:publishReleasePublicationToGitHubPackagesRepository" })
}
