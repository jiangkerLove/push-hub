<template>
  <div class="login-page">
    <div class="login-bg">
      <div class="blob blob-1" />
      <div class="blob blob-2" />
      <div class="blob blob-3" />
    </div>

    <div class="login-shell">
      <section class="login-hero">
        <div class="hero-badge">Push Hub</div>
        <h1>{{ needsSetup ? '欢迎使用 Push Hub' : '把推送做得<br />更轻、更快、更好玩' }}</h1>
        <p>
          {{
            needsSetup
              ? '首次部署请先创建管理员账号，登录后再创建应用并配置厂商凭证。'
              : '在线通道、厂商降级、模板推送，一个控制台全部搞定。'
          }}
        </p>
        <div class="hero-tags">
          <span>WebSocket 在线</span>
          <span>模板化</span>
          <span>链路追踪</span>
        </div>
      </section>

      <el-card class="login-card" shadow="never" v-loading="bootstrapping">
        <div class="login-card-head">
          <div class="login-icon">{{ needsSetup ? '✨' : '👋' }}</div>
          <div>
            <h2>{{ needsSetup ? '创建管理员' : '欢迎回来' }}</h2>
            <p>{{ needsSetup ? '设置后即可进入控制台' : '登录 Push Hub 管理端' }}</p>
          </div>
        </div>

        <el-alert
          v-if="loginHint"
          :title="loginHint"
          type="info"
          show-icon
          :closable="false"
          class="login-hint"
        />

        <el-form :model="form" label-position="top" @submit.prevent="onSubmit">
          <el-form-item label="用户名">
            <el-input
              v-model="form.username"
              autocomplete="username"
              :placeholder="needsSetup ? '3–32 位，字母数字下划线' : '输入管理员账号'"
              size="large"
            />
          </el-form-item>
          <el-form-item label="密码">
            <el-input
              v-model="form.password"
              type="password"
              show-password
              autocomplete="current-password"
              :placeholder="needsSetup ? '至少 6 位' : '输入密码'"
              size="large"
              @keyup.enter="onSubmit"
            />
          </el-form-item>
          <el-form-item v-if="needsSetup" label="确认密码">
            <el-input
              v-model="form.confirmPassword"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="再次输入密码"
              size="large"
              @keyup.enter="onSubmit"
            />
          </el-form-item>
          <el-button
            type="primary"
            size="large"
            round
            :loading="auth.loading"
            class="login-btn"
            @click="onSubmit"
          >
            {{ needsSetup ? '创建并进入' : '进入控制台' }}
          </el-button>
        </el-form>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { fetchBootstrapStatus } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { useAppStore } from '@/stores/app'

const auth = useAuthStore()
const appStore = useAppStore()
const router = useRouter()
const route = useRoute()

const bootstrapping = ref(true)
const needsSetup = ref(false)

const loginHint = computed(() => {
  if (route.query.reason === 'password_changed') {
    return '密码已更新，请使用新密码重新登录。'
  }
  if (route.query.reason === 'session_expired') {
    return '登录状态已失效，请重新登录。'
  }
  return ''
})

const form = reactive({
  username: '',
  password: '',
  confirmPassword: '',
})

onMounted(async () => {
  try {
    const status = await fetchBootstrapStatus()
    needsSetup.value = status.needs_setup
  } catch (error) {
    ElMessage.error(
      error instanceof Error ? error.message : '无法连接服务端，请确认 push-hub server 已启动（默认 http://127.0.0.1:3000）',
    )
  } finally {
    bootstrapping.value = false
  }
})

function isRedirectValid(path: string) {
  const appId = path.match(/^\/apps\/([^/]+)/)?.[1]
  if (!appId) return true
  return appStore.isKnownAppId(appId)
}

async function onSubmit() {
  const username = form.username.trim()
  const password = form.password
  if (!username || !password) {
    ElMessage.warning('请填写用户名和密码')
    return
  }

  if (needsSetup.value) {
    if (password.length < 6) {
      ElMessage.warning('密码至少 6 位')
      return
    }
    if (password !== form.confirmPassword) {
      ElMessage.warning('两次输入的密码不一致')
      return
    }
  }

  try {
    if (needsSetup.value) {
      await auth.setup(username, password)
      ElMessage.success('管理员已创建')
    } else {
      await auth.login(username, password)
      ElMessage.success('登录成功')
    }

    const redirect = route.query.redirect as string | undefined
    await appStore.loadApps()
    if (redirect && isRedirectValid(redirect)) {
      router.push(redirect)
      return
    }
    router.push(appStore.resolveDefaultAppPath())
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : needsSetup.value ? '创建失败' : '登录失败')
  }
}
</script>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 20px;
  position: relative;
  overflow: hidden;
  background: var(--ph-bg-soft);
}

.login-bg {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(40px);
  opacity: 0.55;
}

.blob-1 {
  width: 420px;
  height: 420px;
  top: -120px;
  left: -80px;
  background: #c7d2fe;
}

.blob-2 {
  width: 360px;
  height: 360px;
  right: -100px;
  bottom: -80px;
  background: #fbcfe8;
}

.blob-3 {
  width: 240px;
  height: 240px;
  top: 40%;
  left: 45%;
  background: #bfdbfe;
}

.login-shell {
  position: relative;
  z-index: 1;
  width: min(960px, 100%);
  display: grid;
  grid-template-columns: 1.1fr 0.9fr;
  gap: 28px;
  align-items: center;
}

.login-hero {
  padding: 12px 8px;
}

.hero-badge {
  display: inline-flex;
  padding: 6px 12px;
  margin-bottom: 18px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.75);
  border: 1px solid var(--ph-border);
  color: var(--ph-primary-dark);
  font-size: 13px;
  font-weight: 700;
}

.login-hero h1 {
  margin: 0 0 14px;
  font-size: clamp(32px, 5vw, 44px);
  line-height: 1.15;
  letter-spacing: -0.03em;
  font-weight: 900;
  color: var(--ph-text);
}

.login-hero p {
  margin: 0;
  max-width: 420px;
  font-size: 16px;
  line-height: 1.7;
  color: var(--ph-text-muted);
}

.hero-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 24px;
}

.hero-tags span {
  padding: 8px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid var(--ph-border);
  color: var(--ph-primary-dark);
  font-size: 13px;
  font-weight: 600;
}

.login-card {
  padding: 8px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.8);
  box-shadow: var(--ph-shadow-md);
}

.login-card-head {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 24px;
}

.login-icon {
  width: 52px;
  height: 52px;
  border-radius: 16px;
  display: grid;
  place-items: center;
  font-size: 24px;
  background: var(--ph-gradient-soft);
}

.login-card-head h2 {
  margin: 0;
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
}

.login-card-head p {
  margin: 4px 0 0;
  color: var(--ph-text-muted);
  font-size: 14px;
}

.login-hint {
  margin-bottom: 16px;
}

.login-btn {
  width: 100%;
  margin-top: 8px;
  height: 46px;
  font-size: 15px;
  font-weight: 700;
}

@media (max-width: 860px) {
  .login-shell {
    grid-template-columns: 1fr;
    max-width: 440px;
  }

  .login-hero {
    text-align: center;
  }

  .login-hero p,
  .hero-tags {
    margin-left: auto;
    margin-right: auto;
    justify-content: center;
  }
}
</style>
