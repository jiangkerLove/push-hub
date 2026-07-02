pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
        maven { url = uri("https://developer.huawei.com/repo/") }
    }
}
plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

// fat-aar 的 embed 需 Maven 坐标；AGP 禁止 library 直接 embed 本地 files()，故构建前同步到 vendor-maven（仅本机构建用，不提交）
val pushHubGroupId = "com.jiangker.push"
val vendorSdks = listOf(
    Triple("push-xiaomi", "thirdparty-xiaomi", "7.12.4") to "MiPush_SDK_Client_7_12_4-C_3rd.aar",
    Triple("push-oppo", "thirdparty-oppo", "3.7.1") to "com.heytap.msp_V3.7.1.aar",
    Triple("push-vivo", "thirdparty-vivo", "4.1.5.0") to "vpush_clientSdk_v4.1.5.0_515.aar",
    Triple("push-honor", "thirdparty-honor", "10.0.13.305") to "push-10.0.13.305.aar",
    Triple("push-meizu", "thirdparty-meizu", "5.0.5") to "push-internal-5.0.5.aar",
)
val vendorMavenDir = rootDir.resolve("vendor-maven")
vendorSdks.forEach { (coords, fileName) ->
    val (module, artifactId, version) = coords
    val src = rootDir.resolve("$module/libs/$fileName")
    if (!src.exists()) {
        println("WARNING: missing vendor AAR $src")
        return@forEach
    }
    val destDir = vendorMavenDir.resolve("${pushHubGroupId.replace('.', '/')}/$artifactId/$version")
    destDir.mkdirs()
    val aarDest = destDir.resolve("$artifactId-$version.aar")
    if (!aarDest.exists() || aarDest.length() != src.length() || aarDest.lastModified() < src.lastModified()) {
        src.copyTo(aarDest, overwrite = true)
    }
    destDir.resolve("$artifactId-$version.pom").writeText(
        """
        |<?xml version="1.0" encoding="UTF-8"?>
        |<project xmlns="http://maven.apache.org/POM/4.0.0">
        |  <modelVersion>4.0.0</modelVersion>
        |  <groupId>$pushHubGroupId</groupId>
        |  <artifactId>$artifactId</artifactId>
        |  <version>$version</version>
        |  <packaging>aar</packaging>
        |</project>
        """.trimMargin(),
    )
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven { url = uri(vendorMavenDir) }
        maven { url = uri("https://developer.huawei.com/repo/") }
    }
}

rootProject.name = "push-hub"
include(":sample")
include(":push")
include(":push-core")
include(":push-xiaomi")
include(":push-huawei")
include(":push-oppo")
include(":push-vivo")
include(":push-honor")
include(":push-meizu")
