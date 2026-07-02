<template>
  <el-container class="layout">
    <el-aside width="240px" class="aside">
      <div class="brand">
        <div class="brand-icon">P</div>
        <div class="brand-text">
          <span class="brand-name">Push Hub</span>
          <span class="brand-tagline">推送控制台</span>
        </div>
      </div>

      <nav class="nav">
        <router-link to="/apps" class="nav-item" :class="{ active: isAppsManage }">
          <el-icon><Grid /></el-icon>
          <span>应用管理</span>
        </router-link>

        <template v-if="activeAppId">
          <div class="nav-group-label">推送</div>
          <router-link
            v-for="item in pushNavItems"
            :key="item.section"
            :to="`/apps/${activeAppId}/${item.section}`"
            class="nav-item"
            :class="{ active: activeSection === item.section }"
          >
            <el-icon><component :is="item.icon" /></el-icon>
            <span>{{ item.label }}</span>
          </router-link>

          <div class="nav-group-label">分析</div>
          <router-link
            :to="`/apps/${activeAppId}/stats`"
            class="nav-item"
            :class="{ active: activeSection === 'stats' }"
          >
            <el-icon><DataAnalysis /></el-icon>
            <span>推送统计</span>
          </router-link>
          <router-link
            :to="`/apps/${activeAppId}/jobs`"
            class="nav-item"
            :class="{ active: activeSection === 'jobs' }"
          >
            <el-icon><List /></el-icon>
            <span>推送记录</span>
          </router-link>
        </template>

        <div class="nav-group-label">设置</div>
        <template v-if="activeAppId">
          <router-link
            :to="`/apps/${activeAppId}/config`"
            class="nav-item"
            :class="{ active: activeSection === 'config' }"
          >
            <el-icon><Setting /></el-icon>
            <span>应用配置</span>
          </router-link>
          <router-link
            :to="`/apps/${activeAppId}/integrate`"
            class="nav-item"
            :class="{ active: activeSection === 'integrate' }"
          >
            <el-icon><Guide /></el-icon>
            <span>接入指南</span>
          </router-link>
        </template>
        <router-link to="/accounts" class="nav-item" :class="{ active: isAccountsManage }">
          <el-icon><User /></el-icon>
          <span>账号管理</span>
        </router-link>
      </nav>

      <div class="aside-footer">
        <div class="aside-blob aside-blob--1" />
        <div class="aside-blob aside-blob--2" />
      </div>
    </el-aside>

    <el-container class="content-wrap" direction="vertical">
      <el-header class="header">
        <div class="header-left">
          <el-popover
            v-if="currentApp"
            v-model:visible="switcherOpen"
            trigger="click"
            placement="bottom-start"
            :width="220"
            popper-class="app-switcher-popper"
            :show-arrow="false"
            :offset="4"
          >
            <template #reference>
              <button
                type="button"
                class="app-switcher-trigger"
                :class="{ 'is-open': switcherOpen }"
              >
                <div
                  class="app-switcher-trigger__avatar"
                  :style="{ background: appAvatarStyle(currentApp.name) }"
                >
                  {{ currentApp.name.slice(0, 1).toUpperCase() }}
                </div>
                <div class="app-switcher-trigger__text">
                  <span class="app-switcher-trigger__name">
                    {{ currentApp.name }}
                  </span>
                </div>
                <div class="app-switcher-trigger__chevron">
                  <el-icon><ArrowDown /></el-icon>
                </div>
              </button>
            </template>

            <div class="app-switcher-panel">
              <button
                v-for="item in appStore.apps"
                :key="item.id"
                type="button"
                class="app-switcher-item"
                :class="{ 'is-active': currentApp.id === item.id }"
                @click="selectApp(item.id)"
              >
                <div
                  class="app-switcher-item__avatar"
                  :style="{ background: appAvatarStyle(item.name) }"
                >
                  {{ item.name.slice(0, 1).toUpperCase() }}
                </div>
                <div class="app-switcher-item__body">
                  <span class="app-switcher-item__name">{{ item.name }}</span>
                  <span v-if="item.is_default" class="app-switcher-item__badge">默认</span>
                </div>
                <el-icon v-if="currentApp.id === item.id" class="app-switcher-item__check">
                  <Check />
                </el-icon>
              </button>
            </div>
          </el-popover>

          <div v-else class="header-title">{{ pageTitle }}</div>
        </div>

        <div class="header-actions">
          <div class="user-pill">
            <el-icon><UserFilled /></el-icon>
            <span>{{ auth.username }}</span>
          </div>
          <el-button round @click="onLogout">
            <el-icon><SwitchButton /></el-icon>
            退出
          </el-button>
        </div>
      </el-header>
      <el-main class="main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowDown,
  Check,
  Connection,
  DataAnalysis,
  Document,
  Grid,
  Guide,
  Iphone,
  List,
  Promotion,
  Setting,
  SwitchButton,
  User,
  UserFilled,
} from '@element-plus/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { useAppStore } from '@/stores/app'
import { useTimezoneStore } from '@/stores/timezone'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const appStore = useAppStore()
const timezoneStore = useTimezoneStore()
const switcherOpen = ref(false)

const APP_AVATAR_GRADIENTS = [
  'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
  'linear-gradient(135deg, #0ea5e9 0%, #6366f1 100%)',
  'linear-gradient(135deg, #10b981 0%, #0ea5e9 100%)',
  'linear-gradient(135deg, #f59e0b 0%, #ef4444 100%)',
  'linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%)',
  'linear-gradient(135deg, #14b8a6 0%, #6366f1 100%)',
]

function appAvatarStyle(name: string) {
  const index = Math.abs(name.charCodeAt(0)) % APP_AVATAR_GRADIENTS.length
  return APP_AVATAR_GRADIENTS[index]
}

const pushNavItems = [
  { section: 'send', label: '发送测试', icon: Promotion },
  { section: 'templates', label: '推送模板', icon: Document },
  { section: 'channels', label: '推送通道', icon: Connection },
  { section: 'devices', label: '设备列表', icon: Iphone },
] as const

const activeAppId = computed(() => {
  if (typeof route.params.id === 'string') return route.params.id
  return appStore.lastAppId || null
})

const activeSection = computed(() =>
  typeof route.meta.section === 'string' ? route.meta.section : '',
)

const isAppsManage = computed(() => route.name === 'apps')
const isAccountsManage = computed(() => route.name === 'accounts')

const pageTitle = computed(() => (isAccountsManage.value ? '账号管理' : '应用管理'))

const currentApp = computed(() => appStore.currentApp)

function syncSelectedFromRoute() {
  const id = route.params.id
  if (typeof id !== 'string' || !appStore.isKnownAppId(id)) return
  appStore.rememberApp(id)
}

function redirectIfInvalidAppRoute() {
  const id = route.params.id
  if (typeof id !== 'string' || !route.meta.section) return
  if (appStore.isKnownAppId(id)) return
  router.replace(appStore.resolveDefaultAppPath(activeSection.value || 'send'))
}

function selectApp(id: string) {
  switcherOpen.value = false
  onAppChange(id)
}

function onAppChange(id: string) {
  appStore.rememberApp(id)
  const section = activeSection.value || 'send'
  router.push(`/apps/${id}/${section}`)
}

onMounted(async () => {
  try {
    await auth.loadProfile()
    timezoneStore.syncFromProfile(auth.displayTimeZone)
    await appStore.loadApps()
    redirectIfInvalidAppRoute()
    syncSelectedFromRoute()
  } catch {
    auth.logout()
    router.push('/login')
  }
})

watch(
  () => [route.name, route.params.id] as const,
  () => {
    redirectIfInvalidAppRoute()
    syncSelectedFromRoute()
  },
)

function onLogout() {
  auth.logout()
  router.push('/login')
}
</script>

<style scoped>
.layout {
  height: 100vh;
  overflow: hidden;
  background: var(--ph-bg);
}

.aside {
  position: relative;
  height: 100vh;
  flex-shrink: 0;
  overflow: hidden;
  background: linear-gradient(180deg, #ffffff 0%, #f8faff 100%);
  border-right: 1px solid var(--ph-border);
  display: flex;
  flex-direction: column;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 24px 20px 16px;
}

.brand-icon {
  width: 42px;
  height: 42px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  font-size: 18px;
  font-weight: 800;
  color: #fff;
  background: var(--ph-gradient);
  box-shadow: 0 10px 24px rgba(99, 102, 241, 0.35);
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.brand-name {
  font-size: 18px;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--ph-text);
}

.brand-tagline {
  font-size: 12px;
  color: var(--ph-text-muted);
}

.nav {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 4px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-group-label {
  padding: 14px 14px 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--ph-text-muted);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 14px;
  border-radius: 12px;
  color: var(--ph-text-muted);
  text-decoration: none;
  font-weight: 600;
  font-size: 14px;
  transition:
    background 0.18s ease,
    color 0.18s ease,
    transform 0.18s ease;
}

.nav-item:hover {
  background: rgba(99, 102, 241, 0.08);
  color: var(--ph-primary-dark);
  transform: translateX(2px);
}

.nav-item.active {
  background: var(--ph-gradient);
  color: #fff;
  box-shadow: 0 10px 24px rgba(99, 102, 241, 0.28);
}

.aside-footer {
  position: relative;
  height: 100px;
  overflow: hidden;
  flex-shrink: 0;
}

.aside-blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(24px);
  opacity: 0.45;
}

.aside-blob--1 {
  width: 120px;
  height: 120px;
  right: -20px;
  bottom: -20px;
  background: #c4b5fd;
}

.aside-blob--2 {
  width: 80px;
  height: 80px;
  left: 10px;
  bottom: 10px;
  background: #fbcfe8;
}

.content-wrap {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow: hidden;
}

.header {
  flex-shrink: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: auto;
  min-height: 72px;
  padding: 12px 28px;
  overflow: visible;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--ph-border);
}

.header-left {
  display: flex;
  align-items: center;
  min-width: 0;
  flex: 1;
}

.app-switcher-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 220px;
  max-width: 100%;
  padding: 5px 8px 5px 5px;
  border: 1px solid var(--ph-border);
  border-radius: 10px;
  background: #fff;
  box-shadow: var(--ph-shadow-sm);
  cursor: pointer;
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.app-switcher-trigger:hover,
.app-switcher-trigger.is-open {
  border-color: rgba(99, 102, 241, 0.28);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.08);
}

.app-switcher-trigger:focus-visible {
  outline: none;
  border-color: rgba(99, 102, 241, 0.35);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.12);
}

.app-switcher-trigger__avatar {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  display: grid;
  place-items: center;
  font-size: 12px;
  font-weight: 800;
  color: #fff;
  flex-shrink: 0;
}

.app-switcher-trigger__text {
  min-width: 0;
  flex: 1;
  text-align: left;
}

.app-switcher-trigger__name {
  font-size: 13px;
  font-weight: 700;
  color: var(--ph-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-switcher-trigger__chevron {
  width: 22px;
  height: 22px;
  border-radius: 5px;
  display: grid;
  place-items: center;
  color: var(--ph-text-muted);
  background: rgba(99, 102, 241, 0.06);
  flex-shrink: 0;
  transition: transform 0.2s ease;
}

.app-switcher-trigger.is-open .app-switcher-trigger__chevron {
  transform: rotate(180deg);
  color: var(--ph-primary-dark);
}

.header-title {
  font-size: 20px;
  font-weight: 800;
  letter-spacing: -0.02em;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  margin-left: 16px;
}

.user-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: 999px;
  background: #fff;
  border: 1px solid var(--ph-border);
  color: var(--ph-text);
  font-size: 14px;
  font-weight: 600;
}

.main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 20px 28px;
  display: flex;
  flex-direction: column;
}

.main :deep(> *) {
  flex: 1;
  min-height: 0;
}
</style>

<style>
.app-switcher-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  text-align: left;
  transition: background 0.15s ease;
}

.app-switcher-item:hover {
  background: rgba(99, 102, 241, 0.06);
}

.app-switcher-item.is-active {
  background: var(--ph-primary-light);
}

.app-switcher-item__avatar {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  display: grid;
  place-items: center;
  font-size: 12px;
  font-weight: 800;
  color: #fff;
  flex-shrink: 0;
}

.app-switcher-item__body {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.app-switcher-item__name {
  font-size: 13px;
  font-weight: 600;
  color: var(--ph-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-switcher-item__badge {
  flex-shrink: 0;
  padding: 0 5px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  line-height: 16px;
  color: #b45309;
  background: rgba(245, 158, 11, 0.12);
}

.app-switcher-item__check {
  flex-shrink: 0;
  font-size: 14px;
  color: var(--ph-primary);
}
</style>
