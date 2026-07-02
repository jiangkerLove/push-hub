plugins {
    alias(libs.plugins.android.library)
}

apply(plugin = "com.kezong.fat-aar")

android {
    namespace = "com.jiangker.push.huawei"
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

/**
 * sample 通过 files(*.aar) 接入时无 Maven 传递依赖，须 fat-aar 内嵌 push 的 HMS 闭包。
 * opendevice 提供 HmsInstanceId（aaid）；与 push 有重复 string 资源，见 gradle.properties。
 */
val hmsEmbedArtifacts = listOf(
    "com.huawei.hms:push:6.13.0.301",
    "com.huawei.hms:opendevice:6.13.0.301",
    "com.huawei.hms:base:6.13.0.303",
    "com.huawei.hms:device:6.13.0.303",
    "com.huawei.hms:ui:6.13.0.303",
    "com.huawei.hms:log:6.13.0.303",
    "com.huawei.hms:stats:6.13.0.303",
    "com.huawei.hms:hatool:6.13.0.303",
    "com.huawei.hms:baselegacyapi:6.13.0.303",
    "com.huawei.hms:availableupdate:6.13.0.303",
    "com.huawei.hms:network-grs:8.0.1.304",
    "com.huawei.hms:network-common:8.0.1.304",
    "com.huawei.hms:network-framework-compat:8.0.1.304",
    "com.huawei.hmf:tasks:1.5.2.301",
    "com.huawei.android.hms:security-base:1.3.0.301",
    "com.huawei.android.hms:security-encrypt:1.3.0.301",
    "com.huawei.android.hms:security-ssl:1.3.0.301",
    "com.huawei.agconnect:agconnect-core:1.9.1.304",
)

dependencies {
    api(project(":push-core"))
    hmsEmbedArtifacts.forEach { coordinate ->
        add("embed", coordinate)
    }
}
