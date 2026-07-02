import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { updateDisplayTimeZone } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import {
  buildTimeZoneOptions,
  detectBrowserTimeZone,
  formatTimeZoneLabel,
  isValidTimeZone,
} from '@/utils/timezone'

const DEFAULT_TIME_ZONE = 'UTC'

export const useTimezoneStore = defineStore('timezone', () => {
  const timeZone = ref(DEFAULT_TIME_ZONE)
  const saving = ref(false)

  const timeZoneLabel = computed(() => formatTimeZoneLabel(timeZone.value))
  const options = computed(() => buildTimeZoneOptions())

  function syncFromProfile(displayTimeZone: string | null | undefined) {
    timeZone.value = displayTimeZone?.trim() || DEFAULT_TIME_ZONE
  }

  function suggestInitialTimeZone() {
    return detectBrowserTimeZone()
  }

  async function setTimeZone(next: string) {
    if (!isValidTimeZone(next)) return

    const auth = useAuthStore()
    if (!auth.isOwner) return

    saving.value = true
    try {
      const profile = await updateDisplayTimeZone(next)
      auth.applyProfile(profile)
      syncFromProfile(profile.display_time_zone)
    } finally {
      saving.value = false
    }
  }

  return {
    timeZone,
    saving,
    timeZoneLabel,
    options,
    syncFromProfile,
    suggestInitialTimeZone,
    setTimeZone,
  }
})
