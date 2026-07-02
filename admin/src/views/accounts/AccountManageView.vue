<template>
  <div class="ph-page ph-page--fill account-page">
    <div class="ph-page-header">
      <div class="ph-page-header__main">
        <h1 class="ph-page-title">账号管理</h1>
      </div>
      <el-button v-if="auth.isOwner" type="primary" round size="large" @click="openCreate">
        <el-icon><Plus /></el-icon>
        新建子账号
      </el-button>
    </div>

    <div class="ph-page-stack">
      <div v-if="!profileReady" class="ph-panel ph-panel--fill account-loading" v-loading="true" />

      <div v-else-if="!auth.isOwner" class="ph-panel ph-panel--fill sub-profile">
        <div class="ph-panel__toolbar">
          <div class="panel-title">账号信息</div>
        </div>
        <div class="ph-panel__body ph-panel__body--padded">
          <el-form label-width="72px" class="settings-form">
            <el-form-item label="用户名">
              <el-input v-model="usernameForm.username" autocomplete="username" />
            </el-form-item>
            <el-form-item class="settings-form__actions">
              <el-button type="primary" round :loading="saving" @click="saveUsername">保存</el-button>
              <el-button round @click="openChangePassword">修改密码</el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>

      <div v-else class="ph-panel ph-panel--fill owner-panel">
        <div class="ph-panel__toolbar">
          <div class="panel-title">系统设置</div>
        </div>
        <div class="ph-panel__body ph-panel__body--padded owner-panel__body">
          <el-form label-width="72px" class="settings-form">
            <el-form-item label="时区">
              <el-select
                :model-value="timezoneStore.timeZone"
                class="timezone-select"
                filterable
                :loading="timezoneStore.saving"
                :filter-method="filterTimeZones"
                @update:model-value="onTimeZoneChange"
              >
                <el-option-group
                  v-for="group in visibleTimeZoneGroups"
                  :key="group.label"
                  :label="group.label"
                >
                  <el-option
                    v-for="item in group.options"
                    :key="item.value"
                    :label="item.label"
                    :value="item.value"
                  />
                </el-option-group>
              </el-select>
            </el-form-item>
          </el-form>

          <div class="account-section">
            <div class="account-section__head">
              <span class="account-section__title">管理员账号</span>
              <el-tag type="primary" size="small">{{ users.length }} 个</el-tag>
            </div>
            <el-table
              v-loading="loading"
              :data="users"
              class="account-table"
              empty-text="暂无账号"
            >
              <el-table-column prop="username" label="用户名" min-width="160">
                <template #default="{ row }">
                  <div class="account-name-cell">
                    <div class="account-avatar">{{ row.username.slice(0, 1).toUpperCase() }}</div>
                    <span>{{ row.username }}</span>
                  </div>
                </template>
              </el-table-column>
              <el-table-column label="角色" width="110">
                <template #default="{ row }">
                  <el-tag v-if="row.is_owner" type="warning" size="small">主账号</el-tag>
                  <el-tag v-else type="info" size="small">子账号</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="创建时间" min-width="170">
                <template #default="{ row }">{{ formatDateTime(row.created_at) }}</template>
              </el-table-column>
              <el-table-column label="操作" width="180" fixed="right">
                <template #default="{ row }">
                  <el-button
                    v-if="row.username === auth.username"
                    link
                    type="primary"
                    @click="openChangePassword()"
                  >
                    修改密码
                  </el-button>
                  <template v-else-if="!row.is_owner">
                    <el-button link type="primary" @click="openResetPassword(row)">重置密码</el-button>
                    <el-button link type="danger" @click="onDelete(row)">删除</el-button>
                  </template>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </div>
    </div>

    <el-dialog v-model="createVisible" title="新建子账号" width="520px">
      <el-form :model="createForm" label-width="100px">
        <el-form-item label="用户名" required>
          <el-input v-model="createForm.username" autocomplete="off" />
        </el-form-item>
        <el-form-item label="密码" required>
          <el-input
            v-model="createForm.password"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
        <el-form-item label="确认密码" required>
          <el-input
            v-model="createForm.confirmPassword"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button round @click="createVisible = false">取消</el-button>
        <el-button type="primary" round :loading="saving" @click="onCreate">创建</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="passwordVisible" title="修改密码" width="520px">
      <el-form :model="passwordForm" label-width="100px">
        <el-form-item label="当前密码" required>
          <el-input
            v-model="passwordForm.currentPassword"
            type="password"
            show-password
            autocomplete="current-password"
          />
        </el-form-item>
        <el-form-item label="新密码" required>
          <el-input
            v-model="passwordForm.newPassword"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
        <el-form-item label="确认密码" required>
          <el-input
            v-model="passwordForm.confirmPassword"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button round @click="passwordVisible = false">取消</el-button>
        <el-button type="primary" round :loading="saving" @click="onChangePassword">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="resetVisible"
      :title="resetTarget ? `重置密码 · ${resetTarget.username}` : '重置密码'"
      width="520px"
    >
      <el-form :model="resetForm" label-width="100px">
        <el-form-item label="新密码" required>
          <el-input
            v-model="resetForm.newPassword"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
        <el-form-item label="确认密码" required>
          <el-input
            v-model="resetForm.confirmPassword"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button round @click="resetVisible = false">取消</el-button>
        <el-button type="primary" round :loading="saving" @click="onResetPassword">重置</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import type { AdminUserSummary } from '@/api/types'
import {
  changeMyPassword,
  createAdminUser,
  deleteAdminUser,
  fetchAdminUsers,
  resetAdminUserPassword,
  updateMyUsername,
} from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { useTimezoneStore } from '@/stores/timezone'
import { formatDateTime } from '@/utils/formatDateTime'

const auth = useAuthStore()
const timezoneStore = useTimezoneStore()
const route = useRoute()
const router = useRouter()
const loading = ref(false)
const saving = ref(false)
const users = ref<AdminUserSummary[]>([])

const profileReady = computed(() => auth.username.length > 0)
const timeZoneFilter = ref('')

const createVisible = ref(false)
const passwordVisible = ref(false)
const resetVisible = ref(false)
const resetTarget = ref<AdminUserSummary | null>(null)

const usernameForm = reactive({
  username: '',
})

const createForm = reactive({
  username: '',
  password: '',
  confirmPassword: '',
})

const passwordForm = reactive({
  currentPassword: '',
  newPassword: '',
  confirmPassword: '',
})

const resetForm = reactive({
  newPassword: '',
  confirmPassword: '',
})

const timeZoneGroups = computed(() => {
  const groups = new Map<string, { label: string; value: string }[]>()
  for (const item of timezoneStore.options) {
    const list = groups.get(item.group) ?? []
    list.push({ label: item.label, value: item.value })
    groups.set(item.group, list)
  }
  return [...groups.entries()].map(([label, options]) => ({ label, options }))
})

const visibleTimeZoneGroups = computed(() => {
  const query = timeZoneFilter.value.trim().toLowerCase()
  if (!query) return timeZoneGroups.value
  return timeZoneGroups.value
    .map((group) => ({
      ...group,
      options: group.options.filter(
        (item) =>
          item.label.toLowerCase().includes(query) ||
          item.value.toLowerCase().includes(query),
      ),
    }))
    .filter((group) => group.options.length > 0)
})

watch(
  () => auth.username,
  (username) => {
    usernameForm.username = username
  },
  { immediate: true },
)

function filterTimeZones(query: string) {
  timeZoneFilter.value = query
}

function onTimeZoneChange(value: string) {
  void timezoneStore.setTimeZone(value)
  timeZoneFilter.value = ''
}

function resetCreateForm() {
  createForm.username = ''
  createForm.password = ''
  createForm.confirmPassword = ''
}

function resetPasswordForm() {
  passwordForm.currentPassword = ''
  passwordForm.newPassword = ''
  passwordForm.confirmPassword = ''
}

function resetResetForm() {
  resetForm.newPassword = ''
  resetForm.confirmPassword = ''
}

async function loadUsers() {
  if (!auth.isOwner) return
  loading.value = true
  try {
    users.value = await fetchAdminUsers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载失败')
  } finally {
    loading.value = false
  }
}

watch(
  () => auth.isOwner,
  (isOwner) => {
    if (isOwner) void loadUsers()
  },
  { immediate: true },
)

watch(
  () => route.name,
  (name) => {
    if (name === 'accounts' && auth.isOwner) void loadUsers()
  },
)

function openCreate() {
  resetCreateForm()
  createVisible.value = true
}

function openChangePassword() {
  resetPasswordForm()
  passwordVisible.value = true
}

function openResetPassword(row: AdminUserSummary) {
  resetTarget.value = row
  resetResetForm()
  resetVisible.value = true
}

function validatePasswordPair(password: string, confirmPassword: string) {
  if (password.length < 6) {
    ElMessage.warning('密码至少 6 位')
    return false
  }
  if (password !== confirmPassword) {
    ElMessage.warning('两次输入的密码不一致')
    return false
  }
  return true
}

async function saveUsername() {
  const username = usernameForm.username.trim()
  if (!username) {
    ElMessage.warning('请填写用户名')
    return
  }
  if (username === auth.username) return

  saving.value = true
  try {
    const data = await updateMyUsername(username)
    auth.applySession(data)
    timezoneStore.syncFromProfile(data.display_time_zone)
    ElMessage.success('已更新')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存失败')
  } finally {
    saving.value = false
  }
}

async function onCreate() {
  const username = createForm.username.trim()
  if (!username) {
    ElMessage.warning('请填写用户名')
    return
  }
  if (!validatePasswordPair(createForm.password, createForm.confirmPassword)) return

  saving.value = true
  try {
    await createAdminUser({ username, password: createForm.password })
    ElMessage.success('已创建')
    createVisible.value = false
    await loadUsers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '创建失败')
  } finally {
    saving.value = false
  }
}

async function onChangePassword() {
  if (!passwordForm.currentPassword) {
    ElMessage.warning('请填写当前密码')
    return
  }
  if (!validatePasswordPair(passwordForm.newPassword, passwordForm.confirmPassword)) return

  saving.value = true
  try {
    const result = await changeMyPassword({
      current_password: passwordForm.currentPassword,
      new_password: passwordForm.newPassword,
    })
    passwordVisible.value = false
    if (result.require_relogin) {
      auth.logout()
      await router.push({ name: 'login', query: { reason: 'password_changed' } })
      return
    }
    ElMessage.success('已更新')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '修改失败')
  } finally {
    saving.value = false
  }
}

async function onResetPassword() {
  if (!resetTarget.value) return
  if (!validatePasswordPair(resetForm.newPassword, resetForm.confirmPassword)) return

  saving.value = true
  try {
    await resetAdminUserPassword(resetTarget.value.id, {
      new_password: resetForm.newPassword,
    })
    ElMessage.success('已重置')
    resetVisible.value = false
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '重置失败')
  } finally {
    saving.value = false
  }
}

async function onDelete(row: AdminUserSummary) {
  try {
    await ElMessageBox.confirm(`确定删除子账号「${row.username}」吗？`, '确认删除', {
      type: 'warning',
    })
    await deleteAdminUser(row.id)
    ElMessage.success('已删除')
    await loadUsers()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error instanceof Error ? error.message : '删除失败')
    }
  }
}

onMounted(() => {
  timezoneStore.syncFromProfile(auth.displayTimeZone)
  if (auth.isOwner && !auth.displayTimeZone) {
    void timezoneStore.setTimeZone(timezoneStore.suggestInitialTimeZone())
  }
})
</script>

<style scoped>
.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 700;
  color: var(--ph-text);
}

.settings-form {
  max-width: 520px;
}

.settings-form :deep(.el-form-item:last-child) {
  margin-bottom: 0;
}

.settings-form__actions :deep(.el-form-item__content) {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.timezone-select {
  width: min(100%, 420px);
}

.account-loading {
  min-height: 240px;
}

.owner-panel__body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.account-section {
  margin-top: 8px;
  padding-top: 20px;
  border-top: 1px solid var(--ph-border);
}

.account-section__head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
}

.account-section__title {
  font-size: 15px;
  font-weight: 700;
  color: var(--ph-text);
}

.account-table {
  border: none;
  border-radius: var(--ph-radius-sm);
  overflow: hidden;
}

.account-table :deep(.el-table__inner-wrapper::before) {
  display: none;
}

.account-name-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.account-avatar {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 800;
  color: #fff;
  background: var(--ph-gradient);
}
</style>
