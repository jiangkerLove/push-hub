import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchApps } from '@/api/client'
import type { AppSummary } from '@/api/types'

export const LAST_APP_KEY = 'push_hub_last_app_id'
/** @deprecated 旧版「全部应用」选项，已不再使用 */
export const LEGACY_ALL_APP_ID = '__all__'

export function sanitizeStoredAppId(id: string | null | undefined): string | null {
  if (!id || id === LEGACY_ALL_APP_ID) return null
  return id
}

export function isReservedAppId(id: string) {
  return id === LEGACY_ALL_APP_ID
}

function readStoredAppId() {
  const stored = localStorage.getItem(LAST_APP_KEY)
  const sanitized = sanitizeStoredAppId(stored)
  if (stored && !sanitized) {
    localStorage.removeItem(LAST_APP_KEY)
  }
  return sanitized || ''
}

export const useAppStore = defineStore('app', () => {
  const apps = ref<AppSummary[]>([])
  const loading = ref(false)
  const lastAppId = ref(readStoredAppId())

  const currentApp = computed(
    () => apps.value.find((item) => item.id === lastAppId.value) || null,
  )

  async function loadApps() {
    loading.value = true
    try {
      apps.value = await fetchApps()
      ensureCurrentApp()
      return apps.value
    } finally {
      loading.value = false
    }
  }

  function ensureCurrentApp() {
    if (apps.value.length === 0) {
      lastAppId.value = ''
      localStorage.removeItem(LAST_APP_KEY)
      return null
    }
    if (lastAppId.value && apps.value.some((item) => item.id === lastAppId.value)) {
      return lastAppId.value
    }
    const fallback = apps.value.find((item) => item.is_default) || apps.value[0]
    rememberApp(fallback.id)
    return fallback.id
  }

  function resolveDefaultAppPath(section = 'send') {
    const id = ensureCurrentApp()
    if (id) return `/apps/${id}/${section}`
    return '/apps'
  }

  function rememberApp(id: string) {
    if (!id || isReservedAppId(id)) return
    lastAppId.value = id
    localStorage.setItem(LAST_APP_KEY, id)
  }

  function isKnownAppId(id: string) {
    return apps.value.some((item) => item.id === id)
  }

  return {
    apps,
    loading,
    lastAppId,
    currentApp,
    loadApps,
    rememberApp,
    ensureCurrentApp,
    resolveDefaultAppPath,
    isKnownAppId,
  }
})
