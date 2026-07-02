import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

/** sample/gradle.properties 由 Gradle 自动加载；旧版 push.properties 仍兼容 */
val legacyPushProperties by lazy {
    Properties().apply {
        file("push.properties").takeIf { it.exists() }?.inputStream()?.use { load(it) }
    }
}

fun pushProp(name: String, default: String = ""): String {
    (findProperty(name) as? String)?.trim()?.takeIf { it.isNotEmpty() }?.let { return it }
    legacyPushProperties.getProperty(name)?.trim()?.takeIf { it.isNotEmpty() }?.let { return it }
    return default
}

val pushHubVersion = rootProject.extra["pushHubVersion"] as String

/** sample 与第三方接入一致：使用 libs/ 下的版本化 AAR，勿用 project() 依赖各模块 */
val pushHubAarModules = listOf(
    "push-core",
    "push",
    "push-xiaomi",
    "push-huawei",
    "push-oppo",
    "push-vivo",
    "push-honor",
    "push-meizu",
)

fun pushHubAar(module: String): File = file("libs/$module-$pushHubVersion.aar")

val pushConfig = mapOf(
    "PUSH_HUB_SERVER" to pushProp("PUSH_HUB_SERVER", ""),
    "PUSH_HUB_APP_ID" to pushProp("PUSH_HUB_APP_ID"),
    "HUAWEI_APP_ID" to pushProp("HUAWEI_APP_ID"),
    "XIAOMI_APP_ID" to pushProp("XIAOMI_APP_ID"),
    "XIAOMI_APP_KEY" to pushProp("XIAOMI_APP_KEY"),
    "XIAOMI_CHANNEL_ID" to pushProp("XIAOMI_CHANNEL_ID"),
    "OPPO_APP_KEY" to pushProp("OPPO_APP_KEY"),
    "OPPO_APP_SECRET" to pushProp("OPPO_APP_SECRET"),
    "VIVO_APP_ID" to pushProp("VIVO_APP_ID"),
    "VIVO_APP_KEY" to pushProp("VIVO_APP_KEY"),
    "HONOR_APP_ID" to pushProp("HONOR_APP_ID"),
    "MEIZU_APP_ID" to pushProp("MEIZU_APP_ID"),
    "MEIZU_APP_KEY" to pushProp("MEIZU_APP_KEY"),
)

android {
    namespace = "com.jiangker.push.sample"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        applicationId = "com.jiangker.push.sample"
        minSdk = 23
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        manifestPlaceholders += pushConfig
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
        debug {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
}

tasks.named("preBuild") {
    doFirst {
        val missingAars = pushHubAarModules
            .map(::pushHubAar)
            .filterNot { it.exists() }
        require(missingAars.isEmpty()) {
            """
            缺少 Push Hub AAR：${missingAars.joinToString { it.name }}
            - 从 GitHub Release 下载对应 AAR 放入 sample/libs/
            - 或本地构建：./gradlew syncPushHubAarsToSample
            """.trimIndent()
        }

        if (!file("gradle.properties").exists() && pushProp("PUSH_HUB_APP_ID").isBlank()) {
            logger.warn(
                """
                未找到 sample/gradle.properties，且 PUSH_HUB_APP_ID 为空。
                请执行：cp sample/gradle.properties.example sample/gradle.properties
                """.trimIndent(),
            )
        }
    }
}

dependencies {
    pushHubAarModules.forEach { module ->
        implementation(files(pushHubAar(module)))
    }
    implementation(libs.okhttp)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.tooling)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}
