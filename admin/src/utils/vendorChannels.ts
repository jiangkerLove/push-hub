/** 各厂商内置通道 / 分类，用于按厂商管理通道。模板选择仍用已启用通道列表。 */

export type VendorPlatform = 'xiaomi' | 'huawei' | 'oppo' | 'vivo' | 'honor' | 'meizu'

/** 公信默认可用，私信需勾选启用（vivo / 华为） */
export type CategoryLevel = 'public' | 'private'

/** 公信默认 / 私信勾选（vivo / 华为） */
export type PublicPrivatePlatform = 'vivo' | 'huawei'
/** 公信私信均默认可用（魅族 / OPPO） */
export type AlwaysOnMsgTypePlatform = 'meizu' | 'oppo'
/** 使用内置消息分类的厂商（相对小米等手填通道） */
export type CategoryManagedPlatform = PublicPrivatePlatform | AlwaysOnMsgTypePlatform

export interface BuiltinVendorCategory {
  code: string
  name: string
  level: CategoryLevel
  levelLabel: string
  description?: string
}

export const VENDOR_PLATFORMS: { value: VendorPlatform; label: string }[] = [
  { value: 'xiaomi', label: '小米' },
  { value: 'huawei', label: '华为' },
  { value: 'oppo', label: 'OPPO' },
  { value: 'vivo', label: 'vivo' },
  { value: 'honor', label: '荣耀' },
  { value: 'meizu', label: '魅族' },
]

export const PUBLIC_PRIVATE_PLATFORMS: PublicPrivatePlatform[] = ['vivo', 'huawei']
export const ALWAYS_ON_MSG_TYPE_PLATFORMS: AlwaysOnMsgTypePlatform[] = ['meizu', 'oppo']
export const CATEGORY_MANAGED_PLATFORMS: CategoryManagedPlatform[] = [
  ...PUBLIC_PRIVATE_PLATFORMS,
  ...ALWAYS_ON_MSG_TYPE_PLATFORMS,
]

export function isCategoryManagedPlatform(
  platform: string,
): platform is CategoryManagedPlatform {
  return (CATEGORY_MANAGED_PLATFORMS as string[]).includes(platform)
}

export function isPublicPrivatePlatform(platform: string): platform is PublicPrivatePlatform {
  return (PUBLIC_PRIVATE_PLATFORMS as string[]).includes(platform)
}

export function isAlwaysOnMsgTypePlatform(
  platform: string,
): platform is AlwaysOnMsgTypePlatform {
  return (ALWAYS_ON_MSG_TYPE_PLATFORMS as string[]).includes(platform)
}

/** 荣耀：无需通道 / 分类配置，有推送密钥即可 */
export function isNoChannelPlatform(platform: string): boolean {
  return platform === 'honor'
}

/** vivo：公信=运营消息，私信=系统消息 */
export const VIVO_CATEGORIES: BuiltinVendorCategory[] = [
  {
    code: 'NEWS',
    name: '新闻',
    level: 'public',
    levelLabel: '公信',
    description: '资讯、时事等新闻内容',
  },
  {
    code: 'CONTENT',
    name: '内容推荐',
    level: 'public',
    levelLabel: '公信',
    description: '内容分发、推荐流相关运营推送',
  },
  {
    code: 'MARKETING',
    name: '运营活动',
    level: 'public',
    levelLabel: '公信',
    description: '活动、促销、营销类推送',
  },
  {
    code: 'SOCIAL',
    name: '社交动态',
    level: 'public',
    levelLabel: '公信',
    description: '关注动态、互动等社交运营推送',
  },
  {
    code: 'IM',
    name: '即时消息',
    level: 'private',
    levelLabel: '私信',
    description: '用户间点对点聊天（私信、群聊等）及邮件提醒',
  },
  {
    code: 'ACCOUNT',
    name: '账号与资产',
    level: 'private',
    levelLabel: '私信',
    description: '账号安全、资产变动等与用户强关联的消息',
  },
  {
    code: 'TODO',
    name: '日程待办',
    level: 'private',
    levelLabel: '私信',
    description: '日程、待办、预约等提醒',
  },
  {
    code: 'DEVICE_REMINDER',
    name: '设备信息',
    level: 'private',
    levelLabel: '私信',
    description: '设备状态、连接与硬件相关提醒',
  },
  {
    code: 'ORDER',
    name: '订单与物流',
    level: 'private',
    levelLabel: '私信',
    description: '订单状态、物流配送等交易履约消息',
  },
  {
    code: 'SUBSCRIPTION',
    name: '订阅提醒',
    level: 'private',
    levelLabel: '私信',
    description: '用户主动订阅后的内容更新提醒',
  },
]

/**
 * 华为：资讯营销（公信默认）与服务与通讯（私信勾选）。
 * @see https://developer.huawei.com/consumer/cn/doc/HMSCore-Guides/message-classification-0000001149358835
 */
export const HUAWEI_CATEGORIES: BuiltinVendorCategory[] = [
  {
    code: 'MARKETING',
    name: '资讯营销',
    level: 'public',
    levelLabel: '公信',
    description: '新闻、内容推荐、社交动态、产品促销、运营活动等（未自分类时默认）',
  },
  {
    code: 'IM',
    name: '即时聊天',
    level: 'private',
    levelLabel: '私信',
    description: '社交通讯类消息，需申请服务与通讯自分类权益',
  },
  {
    code: 'VOIP',
    name: '音视频通话',
    level: 'private',
    levelLabel: '私信',
    description: '语音 / 视频通话相关提醒',
  },
  {
    code: 'SUBSCRIPTION',
    name: '订阅',
    level: 'private',
    levelLabel: '私信',
    description: '用户主动订阅后的内容更新提醒',
  },
  {
    code: 'TRAVEL',
    name: '出行',
    level: 'private',
    levelLabel: '私信',
    description: '行程、交通等出行相关提醒',
  },
  {
    code: 'HEALTH',
    name: '健康',
    level: 'private',
    levelLabel: '私信',
    description: '健康、诊疗等相关提醒',
  },
  {
    code: 'WORK',
    name: '工作事项提醒',
    level: 'private',
    levelLabel: '私信',
    description: '工作待办、会议、审批等事项提醒',
  },
  {
    code: 'ACCOUNT',
    name: '账号动态',
    level: 'private',
    levelLabel: '私信',
    description: '账号上下线、认证、会员、余额等动态',
  },
  {
    code: 'EXPRESS',
    name: '订单与物流',
    level: 'private',
    levelLabel: '私信',
    description: '订单状态、物流配送等交易履约消息',
  },
  {
    code: 'FINANCE',
    name: '财务',
    level: 'private',
    levelLabel: '私信',
    description: '支付、到账、账单等财务提醒',
  },
  {
    code: 'DEVICE_REMINDER',
    name: '设备提醒',
    level: 'private',
    levelLabel: '私信',
    description: 'IoT 设备状态、信息、告警等提醒',
  },
  {
    code: 'MAIL',
    name: '邮件',
    level: 'private',
    levelLabel: '私信',
    description: '新邮件提醒（邮箱 / 办公类应用）',
  },
  {
    code: 'PLAY_VOICE',
    name: '语音播报',
    level: 'private',
    levelLabel: '私信',
    description: '需要用户特别注意的语音提醒',
  },
  {
    code: 'CUSTOMER_SERVICE',
    name: '客服消息',
    level: 'private',
    levelLabel: '私信',
    description: '客服会话相关提醒',
  },
]

/** 魅族：仅公信 / 私信两档，均默认可用 */
export const MEIZU_CATEGORIES: BuiltinVendorCategory[] = [
  {
    code: 'PUBLIC',
    name: '公信',
    level: 'public',
    levelLabel: '公信',
    description: '运营、营销等公开消息',
  },
  {
    code: 'PRIVATE',
    name: '私信',
    level: 'private',
    levelLabel: '私信',
    description: '与用户强相关的私密消息',
  },
]

/**
 * OPPO：内容与营销（公信）/ 通讯与服务（私信），全部分类默认可用。
 * 发送主路径看我方模板是否对应 OPPO 审核模板；仅 IM 可不绑模板自由填写。
 * @see https://open.oppomobile.com/documentation/page/info?id=11236
 */
export const OPPO_CATEGORIES: BuiltinVendorCategory[] = [
  {
    code: 'NEWS',
    name: '新闻资讯',
    level: 'public',
    levelLabel: '公信',
    description: '内容与营销：新闻、时事等资讯类消息',
  },
  {
    code: 'CONTENT',
    name: '内容推荐',
    level: 'public',
    levelLabel: '公信',
    description: '内容与营销：内容分发、推荐流等运营推送',
  },
  {
    code: 'MARKETING',
    name: '平台活动',
    level: 'public',
    levelLabel: '公信',
    description: '内容与营销：活动、促销、营销类推送',
  },
  {
    code: 'SOCIAL',
    name: '社交动态',
    level: 'public',
    levelLabel: '公信',
    description: '内容与营销：关注动态、互动等社交运营推送',
  },
  {
    code: 'IM',
    name: '即时聊天、音频、视频通话',
    level: 'private',
    levelLabel: '私信',
    description: '通讯与服务：可自由填写标题与内容，无需对应 OPPO 审核模板',
  },
  {
    code: 'ACCOUNT',
    name: '个人账号与资产变化',
    level: 'private',
    levelLabel: '私信',
    description: '通讯与服务：账号安全、资产变动等，须对应 OPPO 审核模板',
  },
  {
    code: 'DEVICE_REMINDER',
    name: '个人设备提醒',
    level: 'private',
    levelLabel: '私信',
    description: '通讯与服务：设备状态相关提醒，须对应 OPPO 审核模板',
  },
  {
    code: 'ORDER',
    name: '个人订单 / 物流状态变化',
    level: 'private',
    levelLabel: '私信',
    description: '通讯与服务：订单、物流状态变化，须对应 OPPO 审核模板',
  },
  {
    code: 'TODO',
    name: '个人日程 / 待办',
    level: 'private',
    levelLabel: '私信',
    description: '通讯与服务：日程、待办提醒，须对应 OPPO 审核模板',
  },
  {
    code: 'SUBSCRIPTION',
    name: '个人订阅',
    level: 'private',
    levelLabel: '私信',
    description: '通讯与服务：用户订阅更新提醒，须对应 OPPO 审核模板',
  },
]

const CATEGORIES_BY_PLATFORM: Record<CategoryManagedPlatform, BuiltinVendorCategory[]> = {
  vivo: VIVO_CATEGORIES,
  huawei: HUAWEI_CATEGORIES,
  oppo: OPPO_CATEGORIES,
  meizu: MEIZU_CATEGORIES,
}

export function categoriesForPlatform(platform: CategoryManagedPlatform): BuiltinVendorCategory[] {
  return CATEGORIES_BY_PLATFORM[platform]
}

/** 需默认写入通道表的分类：vivo/华为仅公信，魅族/OPPO 全部 */
export function defaultCategoriesFor(platform: CategoryManagedPlatform): BuiltinVendorCategory[] {
  if (isAlwaysOnMsgTypePlatform(platform)) {
    return categoriesForPlatform(platform)
  }
  return categoriesForPlatform(platform).filter((item) => item.level === 'public')
}

export function categoriesByLevel(
  platform: CategoryManagedPlatform,
  level: CategoryLevel,
): BuiltinVendorCategory[] {
  return categoriesForPlatform(platform).filter((item) => item.level === level)
}

export function publicCategoriesFor(platform: PublicPrivatePlatform): BuiltinVendorCategory[] {
  return categoriesForPlatform(platform).filter((item) => item.level === 'public')
}

export function privateCategoriesFor(platform: PublicPrivatePlatform): BuiltinVendorCategory[] {
  return categoriesForPlatform(platform).filter((item) => item.level === 'private')
}

export function findCategory(
  platform: CategoryManagedPlatform,
  code?: string,
): BuiltinVendorCategory | undefined {
  if (!code) return undefined
  const normalized = code.trim().toUpperCase()
  return categoriesForPlatform(platform).find((item) => item.code === normalized)
}

export function isPublicCategory(platform: CategoryManagedPlatform, code?: string): boolean {
  if (isAlwaysOnMsgTypePlatform(platform)) {
    return findCategory(platform, code) != null
  }
  return findCategory(platform, code)?.level === 'public'
}

export function isPrivateCategory(platform: CategoryManagedPlatform, code?: string): boolean {
  if (isAlwaysOnMsgTypePlatform(platform)) return false
  return findCategory(platform, code)?.level === 'private'
}

/** @deprecated use findCategory('vivo', code) */
export function findVivoCategory(code?: string): BuiltinVendorCategory | undefined {
  return findCategory('vivo', code)
}

/** @deprecated use findCategory('huawei', code) */
export function findHuaweiCategory(code?: string): BuiltinVendorCategory | undefined {
  return findCategory('huawei', code)
}

export function isVivoPublicCategory(code?: string): boolean {
  return isPublicCategory('vivo', code)
}

export function isVivoPrivateCategory(code?: string): boolean {
  return isPrivateCategory('vivo', code)
}

/** 内置分类展示：中文名 + 英文 code */
export function formatCategoryLabel(
  preset: BuiltinVendorCategory,
  options?: { includeLevel?: boolean },
): string {
  const includeLevel = options?.includeLevel ?? true
  let namePart = preset.name
  if (
    includeLevel &&
    preset.levelLabel &&
    preset.code !== 'PUBLIC' &&
    preset.code !== 'PRIVATE'
  ) {
    namePart = `${preset.levelLabel} · ${preset.name}`
  }
  return `${namePart} (${preset.code})`
}

/** 通道 / 模板选项展示：优先用内置中文名，并附带英文分类 code */
export function formatChannelCodeLabel(platform: string, code: string, name?: string): string {
  if (isCategoryManagedPlatform(platform)) {
    const preset = findCategory(platform, code)
    if (!preset) return name || code
    return formatCategoryLabel(preset)
  }
  if (name) return name
  return code
}

/**
 * OPPO：仅非 IM 私信须对应审核模板。
 * 发送主要看模板映射；分类仅用于分辨是否 IM。
 */
export function oppoRequiresPrivateTemplate(code?: string): boolean {
  const preset = findCategory('oppo', code)
  return preset?.level === 'private' && preset.code !== 'IM'
}

export function isOppoImCategory(code?: string): boolean {
  return findCategory('oppo', code)?.code === 'IM'
}

/** 自定义通道厂商：在二级目录下自行新增 */
export function isManualChannelPlatform(platform: VendorPlatform): boolean {
  return platform === 'xiaomi'
}
