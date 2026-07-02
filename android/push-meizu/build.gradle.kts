plugins {
    alias(libs.plugins.android.library)
}

apply(plugin = "com.kezong.fat-aar")

android {
    namespace = "com.jiangker.push.meizu"
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
    add("embed", "$groupId:thirdparty-meizu:5.0.5")
}
