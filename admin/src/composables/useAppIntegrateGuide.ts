import { computed, ref, type Ref } from 'vue'
import type { AppConfig, AppInitSnippet, Template, TemplateContentMode } from '@/api/types'
import { extractTemplateVariables } from '@/utils/templateVariables'

export const SDK_VERSION = '0.1.0'

export const ANDROID_VENDOR_MODULES = [
  { id: 'xiaomi', file: 'push-xiaomi', label: '小米' },
  { id: 'huawei', file: 'push-huawei', label: '华为' },
  { id: 'oppo', file: 'push-oppo', label: 'OPPO' },
  { id: 'vivo', file: 'push-vivo', label: 'vivo' },
  { id: 'honor', file: 'push-honor', label: '荣耀' },
  { id: 'meizu', file: 'push-meizu', label: '魅族' },
] as const

export const BACKEND_FIELDS = [
  {
    name: 'Authorization',
    type: '请求头 string',
    required: '必填',
    requiredLevel: 'required',
    desc:
      '鉴权凭证，非 JSON 请求体字段。值为本页上方 Push API Key，'
      + '格式 Authorization: Bearer phk_…；'
      + '也可使用 X-Push-Hub-Api-Key: phk_…（二选一）。'
      + '与管理端登录 JWT 不同，仅用于 POST /api/v1/push。',
  },
  {
    name: 'app_id',
    type: 'string',
    required: '推荐必填',
    requiredLevel: 'recommended',
    desc:
      'Push Hub 应用 ID（UUID 字符串）。须与 API Key 所属应用一致；'
      + '不传时使用服务端默认应用，此时 Key 也必须属于该默认应用。',
  },
  {
    name: 'template_id',
    type: 'string',
    required: '条件必填',
    requiredLevel: 'conditional',
    desc:
      '推送模板 ID（本页「模板管理」列表中可复制）。'
      + '使用模板推送时必填；省略时改为 title + body 直推，'
      + '此时 channels 等配置按应用默认通道生效。',
  },
  {
    name: 'title',
    type: 'string',
    required: '条件必填',
    requiredLevel: 'conditional',
    desc:
      '推送标题。公信模板、私信「自由填写」模板、或无 template_id 直推时必填；'
      + '拼接私信模板一般不需要（由模板 + title_variables 渲染）。',
  },
  {
    name: 'body',
    type: 'string',
    required: '条件必填',
    requiredLevel: 'conditional',
    desc:
      '推送正文。必填条件同 title。'
      + '公信/自由填写场景下由业务后端在发送时传入最终展示文案。',
  },
  {
    name: 'title_variables',
    type: 'object · Record<string, string>',
    required: '条件必填',
    requiredLevel: 'conditional',
    desc:
      '私信「拼接」模板专用。JSON 对象，键为模板标题中的 {{变量名}}，值为字符串，'
      + '如 { "order_id": "12345" }。与 body_variables 分开，避免同名变量冲突。',
  },
  {
    name: 'body_variables',
    type: 'object · Record<string, string>',
    required: '条件必填',
    requiredLevel: 'conditional',
    desc:
      '私信「拼接」模板专用。JSON 对象，键为模板正文中的 {{变量名}}，值为字符串，'
      + '如 { "carrier": "顺丰" }。服务端据此拼出最终 title/body，'
      + '并供 OPPO 等厂商私信模板参数使用。',
  },
  {
    name: 'targets',
    type: 'object',
    required: '必填',
    requiredLevel: 'required',
    desc:
      '推送目标容器对象，必填。内含 device_ids 或 push_tokens（可二选一或同时传）；'
      + '至少包含一个有效目标，否则请求会被拒绝。',
  },
  {
    name: 'targets.device_ids',
    type: 'string[]',
    required: '二选一',
    requiredLevel: 'conditional',
    desc:
      'device_id 字符串数组，推荐方式。按客户端注册后获得的 ID 发送（本页「设备列表」可见），'
      + '如 ["uuid-1", "uuid-2"]。服务端会解析各设备所属厂商并分别路由。',
  },
  {
    name: 'targets.push_tokens',
    type: 'string[]',
    required: '二选一',
    requiredLevel: 'conditional',
    desc:
      '厂商 push token 字符串数组。须同时传 targets.platform（如 "xiaomi"、"huawei"），'
      + '如 ["regId_xxx"]。适用于已知 token、尚未入库 device_id 的调试场景。',
  },
  {
    name: 'targets.platform',
    type: 'string',
    required: '条件必填',
    requiredLevel: 'conditional',
    desc:
      '厂商平台标识字符串。使用 push_tokens 时必填，取值如 xiaomi、huawei、oppo、vivo、honor、meizu；'
      + 'device_ids 模式下无需填写。',
  },
  {
    name: 'click_action',
    type: 'object',
    required: '可选',
    requiredLevel: 'optional',
    desc:
      '点击行为对象。type 为 string 枚举：open_app（默认）、open_page、open_web。'
      + 'open_page 时 activity 为 string（Activity 全类名），可选 params 为 object；'
      + 'open_web 时 url 为 string。示例：{ "type": "open_page", "activity": "com.example.app.MainActivity", "params": { "id": "1" } }。',
  },
  {
    name: 'payload',
    type: 'object',
    required: '可选',
    requiredLevel: 'optional',
    desc:
      '任意 JSON 对象，默认 {}。原样透传给客户端 onMessageReceived，'
      + '如 { "type": "order_shipped", "order_id": "12345" }。'
      + '与 click_action.params 用途不同：payload 给业务逻辑，params 给 Activity Intent。',
  },
  {
    name: 'delivery_mode',
    type: 'string · "notification" | "data"',
    required: '可选',
    requiredLevel: 'optional',
    desc:
      '投递模式字符串，默认 "notification"（通知栏展示）。'
      + '传 "data" 时为透传/数据消息，客户端需自行处理展示；部分厂商通道对 data 有限制。',
  },
  {
    name: 'notify_id',
    type: 'integer',
    required: '可选',
    requiredLevel: 'optional',
    desc:
      '通知栏 ID，JSON 整数（非字符串），范围 0~2147483647，如 1001。'
      + '相同 notify_id 的新消息会覆盖旧通知；不传则不下发该字段。',
  },
  {
    name: 'cache_until',
    type: 'string · ISO 8601',
    required: '可选',
    requiredLevel: 'optional',
    desc:
      '在线消息缓存截止时间，UTC 时间字符串，如 "2026-07-13T08:00:00Z"。'
      + '覆盖模板默认 message_cache_days；设备离线时在此时间前上线仍可收到在线消息。',
  },
  {
    name: 'channels',
    type: 'object',
    required: '可选',
    requiredLevel: 'optional',
    desc:
      '厂商通道配置对象，无 template_id 直推时可选。'
      + '键为平台名（xiaomi、huawei、oppo、vivo、honor、meizu），值为对应配置 object，'
      + '如 { "xiaomi": { "channel_id": "146997" }, "vivo": { "category": "ORDER" } }。'
      + '未传时使用应用「厂商配置」默认值；有 template_id 时以模板 channels 为准。',
  },
] as const

export const ANDROID_DEVICE_ID_SNIPPET = `// 推荐：发送推送时使用唯一 device_id
val deviceId = PushHub.getDeviceId()

// 按平台查询（一般不需要）
val xiaomiId = PushHub.getDeviceId(PushVendor.XIAOMI)`

const ALL_MANIFEST_PLACEHOLDER_KEYS = [
  { key: 'PUSH_HUB_SERVER', section: 'Push Hub' },
  { key: 'PUSH_HUB_APP_ID', section: 'Push Hub' },
  { key: 'XIAOMI_APP_ID', section: '小米' },
  { key: 'XIAOMI_APP_KEY', section: '小米' },
  { key: 'XIAOMI_CHANNEL_ID', section: '小米' },
  { key: 'HUAWEI_APP_ID', section: '华为' },
  { key: 'OPPO_APP_KEY', section: 'OPPO' },
  { key: 'OPPO_APP_SECRET', section: 'OPPO' },
  { key: 'VIVO_APP_ID', section: 'vivo' },
  { key: 'VIVO_APP_KEY', section: 'vivo' },
  { key: 'HONOR_APP_ID', section: '荣耀' },
  { key: 'MEIZU_APP_ID', section: '魅族' },
  { key: 'MEIZU_APP_KEY', section: '魅族' },
] as const

function templateContentMode(template?: Template | null): TemplateContentMode {
  if (!template || template.kind === 'public') return 'compose'
  return template.content_mode ?? 'compose'
}

function isPrivateFreeTemplate(template?: Template | null) {
  return template?.kind === 'private' && templateContentMode(template) === 'free'
}

function escapeKotlinString(value: string) {
  return value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
}

function sampleVariableValue(name: string): string {
  const key = name.toLowerCase()
  if (key.includes('order') || key.includes('no')) return '12345'
  if (key.includes('carrier') || key.includes('express') || key.includes('company')) return '顺丰'
  if (key.includes('track')) return 'SF1234567890'
  if (key.includes('name') || key.includes('user')) return '张三'
  if (key.includes('amount') || key.includes('price')) return '99.00'
  return '示例值'
}

function buildSampleVariableMap(names: string[]): Record<string, string> {
  return Object.fromEntries(names.map((name) => [name, sampleVariableValue(name)]))
}

export function useAppIntegrateGuide(options: {
  appId: Ref<string>
  app: Ref<AppConfig | null>
  templates: Ref<Template[]>
  initSnippet: Ref<AppInitSnippet | null>
}) {
  const { appId, app, templates, initSnippet } = options
  const integrateTab = ref<'android' | 'backend'>('android')

  const integrateServerBaseUrl = computed(() => window.location.origin.replace(/\/$/, ''))

  const pushApiKey = computed(
    () => app.value?.push_api_key || initSnippet.value?.push_api_key || '',
  )

  function filled(...values: Array<string | null | undefined>) {
    return values.every((value) => !!value?.trim())
  }

  /** 根据详情接口实际凭证判断，不依赖列表接口的 has_*（详情响应原先未返回） */
  function vendorConfigured(id: (typeof ANDROID_VENDOR_MODULES)[number]['id']) {
    const a = app.value
    if (!a) return false
    if (id === 'xiaomi') return a.has_xiaomi || filled(a.xiaomi_app_secret)
    if (id === 'huawei') {
      return a.has_huawei || filled(a.huawei_app_id, a.huawei_app_secret)
    }
    if (id === 'oppo') {
      return a.has_oppo || filled(a.oppo_app_key, a.oppo_master_secret)
    }
    if (id === 'vivo') {
      return a.has_vivo || filled(a.vivo_app_id, a.vivo_app_key, a.vivo_app_secret)
    }
    if (id === 'honor') {
      return (
        a.has_honor ||
        filled(a.honor_app_id, a.honor_oauth_client_id, a.honor_app_secret)
      )
    }
    if (id === 'meizu') {
      return a.has_meizu || filled(a.meizu_app_id, a.meizu_app_secret)
    }
    return false
  }

  const configuredVendors = computed(() => {
    return ANDROID_VENDOR_MODULES.filter((item) => vendorConfigured(item.id)).map((item) => ({
      id: item.id,
      label: item.label,
    }))
  })

  const androidDownloadSnippet = computed(() => {
    return `https://github.com/jiangkerLove/push-hub/releases

必选：
- push-core-${SDK_VERSION}.aar
- push-${SDK_VERSION}.aar

厂商模块（按需）：
${ANDROID_VENDOR_MODULES.map(({ file, label }) => `- ${file}-${SDK_VERSION}.aar（${label}）`).join('\n')}

放入 app/libs/`
  })

  function manifestValueFromApp(key: string): string {
    const cfg = app.value
    switch (key) {
      case 'PUSH_HUB_SERVER':
        return integrateServerBaseUrl.value
      case 'PUSH_HUB_APP_ID':
        return appId.value
      case 'XIAOMI_APP_ID':
        return cfg?.xiaomi_app_id || ''
      case 'XIAOMI_APP_KEY':
        return cfg?.xiaomi_app_key || ''
      case 'XIAOMI_CHANNEL_ID':
        return cfg?.xiaomi_channel_id || ''
      case 'HUAWEI_APP_ID':
        return cfg?.huawei_app_id || ''
      case 'OPPO_APP_KEY':
        return cfg?.oppo_app_key || ''
      case 'OPPO_APP_SECRET':
        return cfg?.oppo_app_secret || ''
      case 'VIVO_APP_ID':
        return cfg?.vivo_app_id || ''
      case 'VIVO_APP_KEY':
        return cfg?.vivo_app_key || ''
      case 'HONOR_APP_ID':
        return cfg?.honor_app_id || ''
      case 'MEIZU_APP_ID':
        return cfg?.meizu_app_id || ''
      case 'MEIZU_APP_KEY':
        return cfg?.meizu_app_key || ''
      default:
        return ''
    }
  }

  function buildManifestPlaceholdersKotlin(): string {
    const lines: string[] = []
    let lastSection = ''

    for (const item of ALL_MANIFEST_PLACEHOLDER_KEYS) {
      const value =
        item.key === 'PUSH_HUB_SERVER'
          ? integrateServerBaseUrl.value
          : (() => {
              const raw = initSnippet.value?.manifest_placeholders?.[item.key]
              if (raw != null && String(raw) !== '') return String(raw)
              return manifestValueFromApp(item.key)
            })()

      if (item.section !== lastSection) {
        if (lines.length) lines.push('')
        lines.push(`            // ${item.section}`)
        lastSection = item.section
      }
      lines.push(`            "${item.key}" to "${escapeKotlinString(value)}",`)
    }

    return `android {
    defaultConfig {
        manifestPlaceholders += mapOf(
${lines.join('\n')}
        )
    }
}`
  }

  const androidManifestPlaceholdersSnippet = computed(() => buildManifestPlaceholdersKotlin())

  function toPascalClassPrefix(name?: string | null) {
    const base = (name || 'App').replace(/[\s-_]/g, '').replace(/[^A-Za-z0-9]/g, '')
    if (!base) return 'App'
    return base.charAt(0).toUpperCase() + base.slice(1)
  }

  const androidClassPrefix = computed(() => toPascalClassPrefix(app.value?.name))

  const androidServiceClassName = computed(
    () => `${androidClassPrefix.value}PushService`,
  )

  const androidApplicationClassName = computed(
    () => `${androidClassPrefix.value}Application`,
  )

  const androidApplicationSnippet = computed(() => {
    const appClass = androidApplicationClassName.value
    const serviceClass = androidServiceClassName.value
    return `class ${appClass} : Application() {
    override fun onCreate() {
        super.onCreate()
        PushHub.init(
            context = this,
            config = PushHubConfig.Builder()
                .messageService(${serviceClass}::class.java)
                .build(this),
        )
    }
}`
  })

  const androidDepsSnippet = computed(() => {
    const lines = [
      `    implementation(files("libs/push-core-${SDK_VERSION}.aar"))`,
      `    implementation(files("libs/push-${SDK_VERSION}.aar"))`,
    ]

    for (const { id, file, label } of ANDROID_VENDOR_MODULES) {
      const dep = `    implementation(files("libs/${file}-${SDK_VERSION}.aar"))`
      lines.push(
        vendorConfigured(id) ? dep : `    // ${dep} // ${label}，未开通可保持注释`,
      )
    }

    return `dependencies {
${lines.join('\n')}
}`
  })

  const androidServiceSnippet = computed(() => {
    const className = androidServiceClassName.value
    return `class ${className} : PushMessageService() {
    override fun onNewToken(vendor: PushVendor, token: String) {
        // 厂商 token 更新（SDK 会自动重新注册到服务端）
    }

    override fun onDeviceRegistered(vendor: PushVendor, deviceId: String) {
        // 获得服务端 device_id，可上报业务后端
    }

    override fun onMessageReceived(vendor: PushVendor, message: PushMessage) {
        // 在线 / 厂商通道统一回调
    }

    override fun onError(vendor: PushVendor, errorMessage: String?) {
        // 推送错误
    }
}`
  })

  const androidManifestServiceSnippet = computed(
    () => `<service
    android:name=".${androidServiceClassName.value}"
    android:exported="false" />`,
  )

  const backendApiBase = computed(() => integrateServerBaseUrl.value)

  const backendAuthHeaderSnippet = computed(
    () => `Authorization: Bearer ${pushApiKey.value || 'your_push_api_key'}
X-Push-Hub-Api-Key: ${pushApiKey.value || 'your_push_api_key'}  # 二选一`,
  )

  const sampleTemplateForBackend = computed(() => templates.value[0] ?? null)

  const backendJsonSnippetHint = computed(() => {
    const template = sampleTemplateForBackend.value
    if (!template) {
      return '当前应用尚无模板：示例为无模板直推，传 title 与 body，无需 template_id。'
    }
    if (template.kind === 'public') {
      return `示例按本应用公信模板「${template.name}」：须传 title 与 body（发送时填写）。`
    }
    if (template.kind === 'private' && templateContentMode(template) === 'free') {
      return `示例按本应用私信自由填写模板「${template.name}」：须传 title 与 body。`
    }
    return `示例按本应用私信拼接模板「${template.name}」：须分别传 title_variables 与 body_variables（标题/正文占位符独立替换）。`
  })

  const backendJsonSnippet = computed(() => {
    const body: Record<string, unknown> = {
      app_id: appId.value,
      payload: {
        type: 'order_shipped',
        order_id: '12345',
      },
      delivery_mode: 'notification',
      notify_id: 1001,
      click_action: {
        type: 'open_app',
      },
      targets: {
        device_ids: ['device-uuid-1'],
      },
    }

    const template = sampleTemplateForBackend.value
    if (!template) {
      body.title = '订单已发货'
      body.body = '您的订单已发出，请注意查收。'
      return JSON.stringify(body, null, 2)
    }

    body.template_id = template.id

    if (template.kind === 'public' || isPrivateFreeTemplate(template)) {
      body.title = '订单已发货'
      body.body = '您的订单已发出，请注意查收。'
      return JSON.stringify(body, null, 2)
    }

    const titleVarNames = extractTemplateVariables(template.title)
    const bodyVarNames = extractTemplateVariables(template.body)
    if (titleVarNames.length) {
      body.title_variables = buildSampleVariableMap(titleVarNames)
    }
    if (bodyVarNames.length) {
      body.body_variables = buildSampleVariableMap(bodyVarNames)
    }
    return JSON.stringify(body, null, 2)
  })

  const backendCurlSnippet = computed(() => {
    const key = pushApiKey.value || 'your_push_api_key'
    return `curl -X POST "${backendApiBase.value}/api/v1/push" \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer ${key}" \\
  -d '${backendJsonSnippet.value.replace(/'/g, "'\\''")}'`
  })

  return {
    integrateTab,
    integrateServerBaseUrl,
    pushApiKey,
    configuredVendors,
    androidDownloadSnippet,
    androidDepsSnippet,
    androidManifestPlaceholdersSnippet,
    androidApplicationSnippet,
    androidServiceSnippet,
    androidManifestServiceSnippet,
    androidDeviceIdSnippet: ANDROID_DEVICE_ID_SNIPPET,
    backendFields: BACKEND_FIELDS,
    backendAuthHeaderSnippet,
    backendJsonSnippetHint,
    backendJsonSnippet,
    backendCurlSnippet,
  }
}
