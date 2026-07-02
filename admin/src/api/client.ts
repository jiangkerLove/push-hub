import axios from 'axios'
import type {
  AdminProfile,
  AdminUserSummary,
  AppConfig,
  AppInitSnippet,
  AppSummary,
  BootstrapStatus,
  ChangePasswordRequest,
  CreateAdminUserRequest,
  CreateAppRequest,
  CreatePushChannelRequest,
  CreateTemplateRequest,
  Device,
  LoginResponse,
  PushChannel,
  PushJob,
  PushJobDetail,
  PushStatsOverview,
  ResetAdminUserPasswordRequest,
  SendPushRequest,
  SendPushResponse,
  Template,
  UpdateAppRequest,
  UpdateMyUsernameResponse,
  UpdatePushChannelRequest,
  ValidateAppCredentialsRequest,
  VendorCredentialValidation,
} from './types'

const TOKEN_KEY = 'push_hub_admin_token'

export const api = axios.create({
  baseURL: '/api/v1',
  timeout: 15000,
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem(TOKEN_KEY)
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (
      error.response?.status === 401 &&
      !error.config?.url?.includes('/admin/login') &&
      !error.config?.url?.includes('/admin/bootstrap') &&
      !error.config?.url?.includes('/admin/setup')
    ) {
      localStorage.removeItem(TOKEN_KEY)
      const message = error.response?.data?.error || ''
      const reason =
        typeof message === 'string' && message.includes('session expired')
          ? 'session_expired'
          : undefined
      const query = reason ? `?reason=${reason}` : ''
      window.location.href = `/login${query}`
    }
    const message = error.response?.data?.error || error.message || '请求失败'
    return Promise.reject(new Error(message))
  },
)

export function saveToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token)
}

export function clearToken() {
  localStorage.removeItem(TOKEN_KEY)
}

export function getToken() {
  return localStorage.getItem(TOKEN_KEY)
}

export async function fetchBootstrapStatus() {
  const { data } = await api.get<BootstrapStatus>('/admin/bootstrap')
  return data
}

export async function setupAdmin(username: string, password: string) {
  const { data } = await api.post<LoginResponse>('/admin/setup', { username, password })
  saveToken(data.token)
  return data
}

export async function login(username: string, password: string) {
  const { data } = await api.post<LoginResponse>('/admin/login', { username, password })
  saveToken(data.token)
  return data
}

export async function fetchProfile() {
  const { data } = await api.get<AdminProfile>('/admin/me')
  return data
}

export async function updateDisplayTimeZone(displayTimeZone: string) {
  const { data } = await api.put<AdminProfile>('/admin/settings/display-timezone', {
    display_time_zone: displayTimeZone,
  })
  return data
}

export async function fetchAdminUsers() {
  const { data } = await api.get<AdminUserSummary[]>('/admin/users')
  return data
}

export async function createAdminUser(body: CreateAdminUserRequest) {
  const { data } = await api.post<AdminUserSummary>('/admin/users', body)
  return data
}

export async function deleteAdminUser(id: string) {
  await api.delete(`/admin/users/${id}`)
}

export async function changeMyPassword(body: ChangePasswordRequest) {
  const { data } = await api.put<{ ok: boolean; require_relogin?: boolean }>(
    '/admin/me/password',
    body,
  )
  return data
}

export async function updateMyUsername(username: string) {
  const { data } = await api.put<UpdateMyUsernameResponse>('/admin/me/username', { username })
  saveToken(data.token)
  return data
}

export async function resetAdminUserPassword(id: string, body: ResetAdminUserPasswordRequest) {
  const { data } = await api.put<{ ok: boolean }>(`/admin/users/${id}/password`, body)
  return data
}

export async function fetchApps() {
  const { data } = await api.get<AppSummary[]>('/admin/apps')
  return data
}

export async function fetchApp(id: string) {
  const { data } = await api.get<AppConfig>(`/admin/apps/${id}`)
  return data
}

export async function createApp(body: CreateAppRequest) {
  const { data } = await api.post<AppSummary>('/admin/apps', body)
  return data
}

export async function updateApp(id: string, body: UpdateAppRequest) {
  const { data } = await api.put<AppConfig>(`/admin/apps/${id}`, body)
  return data
}

export async function validateAppCredentials(
  id: string,
  body: ValidateAppCredentialsRequest,
) {
  const { data } = await api.post<{ results: VendorCredentialValidation[] }>(
    `/admin/apps/${id}/validate-credentials`,
    body,
    { timeout: 30000 },
  )
  return data.results
}

export async function deleteApp(id: string) {
  await api.delete(`/admin/apps/${id}`)
}

export async function setDefaultApp(id: string) {
  const { data } = await api.post<AppSummary>(`/admin/apps/${id}/default`)
  return data
}

export async function fetchTemplates(appId: string) {
  const { data } = await api.get<Template[]>(`/admin/apps/${appId}/templates`)
  return data
}

export async function createTemplate(appId: string, body: CreateTemplateRequest) {
  const { data } = await api.post<Template>(`/admin/apps/${appId}/templates`, body)
  return data
}

export async function updateTemplate(id: string, body: CreateTemplateRequest) {
  const { data } = await api.put<Template>(`/admin/templates/${id}`, body)
  return data
}

export async function deleteTemplate(id: string) {
  await api.delete(`/admin/templates/${id}`)
}

export async function fetchDevices(appId: string, limit = 50, offset = 0) {
  const { data } = await api.get<Device[]>(`/admin/apps/${appId}/devices`, {
    params: { limit, offset },
  })
  return data
}

export async function sendPush(appId: string, body: SendPushRequest) {
  const { data } = await api.post<SendPushResponse>(`/admin/apps/${appId}/push`, body)
  return data
}

export async function fetchInitSnippet(appId: string) {
  const { data } = await api.get<AppInitSnippet>(`/admin/apps/${appId}/init-snippet`)
  return data
}

export async function fetchPushStats(appId: string, days = 7) {
  const { data } = await api.get<PushStatsOverview>(`/admin/apps/${appId}/push/stats`, {
    params: { days },
  })
  return data
}

export async function fetchPushJobs(appId: string, limit = 50, offset = 0) {
  const { data } = await api.get<PushJob[]>(`/admin/apps/${appId}/push/jobs`, {
    params: { limit, offset },
  })
  return data
}

export async function fetchPushJobDetail(appId: string, jobId: string) {
  const { data } = await api.get<PushJobDetail>(`/admin/apps/${appId}/push/jobs/${jobId}`)
  return data
}

export async function fetchChannels(appId: string) {
  const { data } = await api.get<PushChannel[]>(`/admin/apps/${appId}/channels`)
  return data
}

export async function createChannel(appId: string, body: CreatePushChannelRequest) {
  const { data } = await api.post<PushChannel>(`/admin/apps/${appId}/channels`, body)
  return data
}

export async function updateChannel(id: string, body: UpdatePushChannelRequest) {
  const { data } = await api.put<PushChannel>(`/admin/channels/${id}`, body)
  return data
}

export async function deleteChannel(id: string) {
  await api.delete(`/admin/channels/${id}`)
}
