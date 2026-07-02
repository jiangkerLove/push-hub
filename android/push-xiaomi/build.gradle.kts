plugins {
    alias(libs.plugins.android.library)
}

apply(plugin = "com.kezong.fat-aar")

android {
    namespace = "com.jiangker.push.xiaomi"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        minSdk = 23
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

val groupId = rootProject.extra["pushHubGroupId"] as String

dependencies {
    api(project(":push-core"))
    // fat-aar 仅识别 Maven 坐标 embed；vendor-maven 由 settings.gradle.kts 从 libs/ 同步
    add("embed", "$groupId:thirdparty-xiaomi:7.12.4")
}
