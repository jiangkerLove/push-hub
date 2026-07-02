export interface LoginResponse {
  token: string
  username: string
}

export interface BootstrapStatus {
  needs_setup: boolean
}

export interface AdminProfile {
  username: string
  is_owner: boolean
  display_time_zone?: string | null
}

export interface AdminUserSummary {
  id: string
  username: string
  is_owner: boolean
  created_at: string
}

export interface CreateAdminUserRequest {
  username: string
  password: string
}

export interface ChangePasswordRequest {
  current_password: string
  new_password: string
}

export interface ResetAdminUserPasswordRequest {
  new_password: string
}

export interface UpdateMyUsernameRequest {
  username: string
}

export interface UpdateMyUsernameResponse {
  token: string
  username: string
  is_owner: boolean
  display_time_zone?: string | null
}

export interface AppSummary {
  id: string
  name: string
  package_name: string
  ios_bundle_id?: string
  harmony_bundle_name?: string
  description?: string
  has_xiaomi: boolean
  has_huawei: boolean
  has_oppo: boolean
  has_vivo: boolean
  has_honor: boolean
  has_meizu: boolean
  online_push_fallback_secs: number
  online_message_cache_secs?: number
  is_default: boolean
  created_at: string
  updated_at: string
}

export interface AppConfig extends AppSummary {
  server_base_url?: string
  push_api_key: string
  xiaomi_app_id?: string
  xiaomi_app_key?: string
  xiaomi_channel_id?: string
  xiaomi_app_secret?: string
  huawei_app_id?: string
  huawei_oauth_client_id?: string
  huawei_app_secret?: string
  oppo_app_key?: string
  oppo_app_secret?: string
  oppo_master_secret?: string
  vivo_app_id?: string
  vivo_app_key?: string
  vivo_app_secret?: string
  honor_app_id?: string
  honor_oauth_client_id?: string
  honor_app_secret?: string
  meizu_app_id?: string
  meizu_app_key?: string
  meizu_app_secret?: string
}

export interface PushChannel {
  id: string
  app_id: string
  platform: 'xiaomi' | 'huawei' | 'oppo' | string
  name: string
  code: string
  description?: string
  is_default: boolean
  created_at: string
  updated_at: string
}

export interface CreatePushChannelRequest {
  platform: string
  name: string
  code: string
  description?: string
  is_default?: boolean
}

export interface UpdatePushChannelRequest extends CreatePushChannelRequest {}

export type TemplateKind = 'public' | 'private'

/** 私信模板内容模式：free=发送时自由填写 / compose=预设文案与变量拼接 */
export type TemplateContentMode = 'free' | 'compose'

export interface Template {
  id: string
  app_id: string
  name: string
  /** public=公信（发送时填标题内容） / private=私信 */
  kind: TemplateKind
  /** 私信内容模式，公信模板可忽略 */
  content_mode?: TemplateContentMode
  title: string
  body: string
  channels: TemplateChannels
  /** 在线消息默认缓存天数（自发送时起算） */
  message_cache_days: number
  created_at: string
  updated_at: string
}

export interface TemplateChannels {
  xiaomi?: { channel_id: string }
  /** 华为消息分类，如 MARKETING / IM / WORK / EXPRESS */
  huawei?: { category: string }
  /** OPPO 消息分类；非 IM 私信需 private_template_id */
  oppo?: { category?: string; channel_id?: string; private_template_id?: string }
  /** vivo 消息二级分类，如 NEWS / IM / ORDER */
  vivo?: { category: string }
  /** 荣耀消息分类，同华为 category */
  honor?: { category: string }
  /** 魅族消息分类：PUBLIC 公信 / PRIVATE 私信（noticeMsgType） */
  meizu?: { msg_type: string }
}

export interface ClickAction {
  type: 'open_app' | 'open_page' | 'open_web'
  activity?: string
  url?: string
  params?: Record<string, unknown>
}

export interface Device {
  id: string
  app_id: string
  package_name: string
  platform: string
  push_token: string
  online_token?: string
  last_online_at?: string
  created_at: string
  updated_at: string
}

export interface SendPushResponse {
  total: number
  success: number
  failed: number
  job_id?: string
  platforms: Array<{
    platform: string
    success: number
    failed: number
    message_id?: string
  }>
}

export interface AppInitSnippet {
  server_base_url: string
  package_name: string
  push_api_key: string
  kotlin: string
  push_properties: string
  manifest_placeholders: Record<string, unknown>
  manifest_placeholders_kotlin: string
}

export interface PushStatsOverview {
  days: number
  total_jobs: number
  total_targets: number
  success_targets: number
  failed_targets: number
  success_rate: number
  push_by_platform: Array<{ platform: string; success: number; failed: number }>
  daily: Array<{ date: string; jobs: number; success: number; failed: number }>
  devices: {
    total: number
    recent_online: number
    new_in_period: number
    by_platform: Array<{ platform: string; count: number }>
  }
  template_count: number
}

export interface PushJob {
  id: string
  app_id: string
  template_id: string
  template_name: string
  title: string
  body: string
  total_targets: number
  success_count: number
  failed_count: number
  batch_id?: string
  created_at: string
}

export interface PushJobTarget {
  id: string
  job_id: string
  device_id?: string
  platform: string
  push_token: string
  route_decision: string
  final_status: string
  final_channel?: string
  outbox_id?: string
  vendor_message_id?: string
  created_at: string
}

export interface PushJobEvent {
  id: string
  job_id: string
  target_id?: string
  stage: string
  status: string
  platform?: string
  detail: string
  metadata?: string
  created_at: string
}

export interface PushOutboxTrace {
  id: string
  push_token: string
  delivered_at?: string
  fallback_sent_at?: string
  fallback_platform?: string
  created_at: string
}

export interface PushJobMessageTrace {
  target: PushJobTarget
  events: PushJobEvent[]
  outbox?: PushOutboxTrace
}

export interface PushJobDetail {
  job: PushJob
  job_events: PushJobEvent[]
  messages: PushJobMessageTrace[]
}

export interface CreateAppRequest {
  name: string
  package_name?: string
  ios_bundle_id?: string
  harmony_bundle_name?: string
  description?: string
  server_base_url?: string
  xiaomi_app_id?: string
  xiaomi_app_key?: string
  xiaomi_channel_id?: string
  xiaomi_app_secret?: string
  huawei_app_id?: string
  huawei_oauth_client_id?: string
  huawei_app_secret?: string
  oppo_app_key?: string
  oppo_app_secret?: string
  oppo_master_secret?: string
  vivo_app_id?: string
  vivo_app_key?: string
  vivo_app_secret?: string
  honor_app_id?: string
  honor_oauth_client_id?: string
  honor_app_secret?: string
  meizu_app_id?: string
  meizu_app_key?: string
  meizu_app_secret?: string
  online_push_fallback_secs?: number
  online_message_cache_secs?: number
}

export interface UpdateAppRequest extends CreateAppRequest {}

export interface ValidateAppCredentialsRequest {
  platform?: string
  package_name?: string
  xiaomi_app_secret?: string
  huawei_app_id?: string
  huawei_oauth_client_id?: string
  huawei_app_secret?: string
  oppo_app_key?: string
  oppo_master_secret?: string
  vivo_app_id?: string
  vivo_app_key?: string
  vivo_app_secret?: string
  honor_app_id?: string
  honor_oauth_client_id?: string
  honor_app_secret?: string
  meizu_app_id?: string
  meizu_app_secret?: string
}

export interface VendorCredentialValidation {
  platform: string
  label: string
  status: 'skipped' | 'incomplete' | 'ok' | 'failed'
  message: string
}

export interface CreateTemplateRequest {
  name: string
  kind?: TemplateKind
  content_mode?: TemplateContentMode
  title?: string
  body?: string
  channels?: TemplateChannels
  message_cache_days?: number
}

export interface UpdateTemplateRequest extends CreateTemplateRequest {}

export interface SendPushRequest {
  template_id?: string
  title?: string
  body?: string
  title_variables?: Record<string, string>
  body_variables?: Record<string, string>
  payload?: Record<string, unknown>
  delivery_mode?: 'notification' | 'data'
  /** 本次推送的点击行为（与消息内容相关，不绑定模板） */
  click_action?: ClickAction
  /** 覆盖模板默认缓存天数对应的截止时间（ISO 8601） */
  cache_until?: string
  /** 通知栏 ID（0~2147483647）；不填则不传给服务端 */
  notify_id?: number
  targets: {
    device_ids: string[]
  }
}
