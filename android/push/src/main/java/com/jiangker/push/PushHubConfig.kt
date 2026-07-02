package com.jiangker.push

import android.content.Context
import com.jiangker.push.core.PushConfigReader
import com.jiangker.push.core.PushMessageService
import com.jiangker.push.core.PushRegistrationListener

class PushHubConfig private constructor(
    val serverBaseUrl: String,
    val pushHubAppId: String,
    val xiaomiAppId: String?,
    val xiaomiAppKey: String?,
    val huaweiAppId: String?,
    val oppoAppKey: String?,
    val oppoAppSecret: String?,
    val vivoAppId: String?,
    val vivoAppKey: String?,
    val honorAppId: String?,
    val meizuAppId: String?,
    val meizuAppKey: String?,
    val messageService: Class<out PushMessageService>,
    val registrationListener: PushRegistrationListener?,
) {
    class Builder {
        private var serverBaseUrl: String? = null
        private var pushHubAppId: String? = null
        private var xiaomiAppId: String? = null
        private var xiaomiAppKey: String? = null
        private var huaweiAppId: String? = null
        private var oppoAppKey: String? = null
        private var oppoAppSecret: String? = null
        private var vivoAppId: String? = null
        private var vivoAppKey: String? = null
        private var honorAppId: String? = null
        private var meizuAppId: String? = null
        private var meizuAppKey: String? = null
        private lateinit var messageServiceClass: Class<out PushMessageService>
        private var registrationListener: PushRegistrationListener? = null

        fun server(baseUrl: String) = apply {
            serverBaseUrl = baseUrl.trim().trimEnd('/')
        }

        /** Push Hub 控制台「应用配置」中的 App ID；未调用时从 Manifest 读取 */
        fun app(appId: String) = apply {
            pushHubAppId = appId.trim()
        }

        /** 显式覆盖 Manifest 中的小米参数 */
        fun xiaomi(appId: String, appKey: String) = apply {
            xiaomiAppId = appId
            xiaomiAppKey = appKey
        }

        /** 显式覆盖 Manifest 中的华为 App ID */
        fun huawei(appId: String) = apply {
            huaweiAppId = appId.trim()
        }

        /** 显式覆盖 Manifest 中的 OPPO 参数 */
        fun oppo(appKey: String, appSecret: String) = apply {
            oppoAppKey = appKey
            oppoAppSecret = appSecret
        }

        /** 显式覆盖 Manifest 中的 vivo 参数 */
        fun vivo(appId: String, appKey: String) = apply {
            vivoAppId = appId
            vivoAppKey = appKey
        }

        /** 显式覆盖 Manifest 中的荣耀 App ID */
        fun honor(appId: String) = apply {
            honorAppId = appId
        }

        /** 显式覆盖 Manifest 中的魅族参数 */
        fun meizu(appId: String, appKey: String) = apply {
            meizuAppId = appId
            meizuAppKey = appKey
        }

        fun messageService(serviceClass: Class<out PushMessageService>) = apply {
            messageServiceClass = serviceClass
        }

        fun registrationListener(listener: PushRegistrationListener) = apply {
            registrationListener = listener
        }

        /**
         * 从 Manifest meta-data 补全未显式设置的 Push Hub 与厂商参数。
         *
         * 请在 App 模块 `defaultConfig.manifestPlaceholders` 中配置占位符，
         * 详见接入指南。
         */
        fun build(context: Context): PushHubConfig {
            check(this::messageServiceClass.isInitialized) {
                "必须调用 messageService() 指定推送消息接收 Service"
            }

            val appContext = context.applicationContext
            val meta = PushConfigReader.loadMeta(appContext)

            val resolvedServer = PushConfigReader.resolveServer(meta, serverBaseUrl)
                ?: error("缺少 Push Hub 服务端地址：请在 manifestPlaceholders 配置 PUSH_HUB_SERVER，或调用 server()")
            val resolvedAppId = PushConfigReader.resolveAppId(meta, pushHubAppId)
                ?: error("缺少 Push Hub 应用 ID：请在 manifestPlaceholders 配置 PUSH_HUB_APP_ID，或调用 app()")

            val xiaomi = PushConfigReader.resolveXiaomi(meta, xiaomiAppId, xiaomiAppKey)
            val huawei = PushConfigReader.resolveHuawei(meta, huaweiAppId)
            val oppo = PushConfigReader.resolveOppo(meta, oppoAppKey, oppoAppSecret)
            val vivo = PushConfigReader.resolveVivo(meta, vivoAppId, vivoAppKey)
            val honor = PushConfigReader.resolveHonor(meta, honorAppId)
            val meizu = PushConfigReader.resolveMeizu(meta, meizuAppId, meizuAppKey)

            return PushHubConfig(
                serverBaseUrl = resolvedServer,
                pushHubAppId = resolvedAppId,
                xiaomiAppId = xiaomi?.appId,
                xiaomiAppKey = xiaomi?.appKey,
                huaweiAppId = huawei?.appId,
                oppoAppKey = oppo?.appKey,
                oppoAppSecret = oppo?.appSecret,
                vivoAppId = vivo?.appId,
                vivoAppKey = vivo?.appKey,
                honorAppId = honor?.appId,
                meizuAppId = meizu?.appId,
                meizuAppKey = meizu?.appKey,
                messageService = messageServiceClass,
                registrationListener = registrationListener,
            )
        }
    }
}
