import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  clearToken,
  fetchProfile,
  getToken,
  login as apiLogin,
  saveToken,
  setupAdmin as apiSetup,
} from '@/api/client'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(getToken())
  const username = ref('')
  const isOwner = ref(false)
  const displayTimeZone = ref<string | null>(null)
  const loading = ref(false)

  function applyProfile(profile: {
    username: string
    is_owner: boolean
    display_time_zone?: string | null
  }) {
    username.value = profile.username
    isOwner.value = profile.is_owner
    displayTimeZone.value = profile.display_time_zone ?? null
  }

  async function login(user: string, password: string) {
    loading.value = true
    try {
      const data = await apiLogin(user, password)
      token.value = data.token
      username.value = data.username
      return data
    } finally {
      loading.value = false
    }
  }

  async function setup(user: string, password: string) {
    loading.value = true
    try {
      const data = await apiSetup(user, password)
      token.value = data.token
      username.value = data.username
      return data
    } finally {
      loading.value = false
    }
  }

  async function loadProfile() {
    if (!token.value) return null
    const profile = await fetchProfile()
    applyProfile(profile)
    return profile
  }

  function applySession(profile: {
    token: string
    username: string
    is_owner: boolean
    display_time_zone?: string | null
  }) {
    token.value = profile.token
    saveToken(profile.token)
    applyProfile(profile)
  }

  function logout() {
    token.value = null
    username.value = ''
    isOwner.value = false
    displayTimeZone.value = null
    clearToken()
  }

  return {
    token,
    username,
    isOwner,
    displayTimeZone,
    loading,
    login,
    setup,
    loadProfile,
    applyProfile,
    applySession,
    logout,
  }
})
