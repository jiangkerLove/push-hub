<template>
  <div class="ph-page ph-page--fill">
    <div class="ph-page-header">
      <div class="ph-page-header__main">
        <h1 class="ph-page-title">应用管理</h1>
        <p class="ph-page-subtitle">新建或删除应用。日常切换请在顶部应用选择器中操作。</p>
      </div>
      <el-button type="primary" round size="large" @click="openCreate">
        <el-icon><Plus /></el-icon>
        新建应用
      </el-button>
    </div>

    <div class="ph-panel ph-panel--fill">
      <div class="ph-panel__toolbar">
        <div class="panel-title">
          <el-icon><Grid /></el-icon>
          <span>已注册应用</span>
          <el-tag type="primary" size="small">{{ appStore.apps.length }} 个</el-tag>
        </div>
        <el-button text @click="loadApps">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>

      <div class="ph-panel__body">
      <div v-if="!appStore.loading && appStore.apps.length === 0" class="empty-apps">
        <h3>还没有应用</h3>
        <p>创建第一个应用后，即可配置厂商凭证、模板，并发送测试推送。</p>
        <el-button type="primary" round size="large" @click="openCreate">
          <el-icon><Plus /></el-icon>
          创建第一个应用
        </el-button>
      </div>
      <el-table v-else v-loading="appStore.loading" :data="appStore.apps" class="app-table">
        <el-table-column prop="name" label="应用名称" min-width="140">
          <template #default="{ row }">
            <div class="app-name-cell">
              <div class="app-avatar">{{ row.name.slice(0, 1).toUpperCase() }}</div>
              <span>{{ row.name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" min-width="180" show-overflow-tooltip />
        <el-table-column label="默认" width="90">
          <template #default="{ row }">
            <el-tag v-if="row.is_default" type="warning" size="small">默认</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="更新时间" min-width="170">
          <template #default="{ row }">{{ formatDateTime(row.updated_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="180" fixed="right">
          <template #default="{ row }">
            <el-button
              v-if="!row.is_default"
              link
              type="primary"
              @click="onSetDefault(row.id)"
            >
              设为默认
            </el-button>
            <el-button
              v-if="!row.is_default"
              link
              type="danger"
              @click="onDelete(row.id)"
            >
              删除
            </el-button>
            <span v-if="row.is_default" class="ph-muted">默认应用不可删除</span>
          </template>
        </el-table-column>
      </el-table>
      </div>
    </div>

    <el-dialog v-model="dialogVisible" title="新建应用" width="560px" class="create-dialog">
      <el-form :model="form" label-width="120px">
        <el-form-item label="应用名称" required>
          <el-input v-model="form.name" placeholder="如：示例应用" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="form.description" type="textarea" :rows="2" placeholder="可选" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button round @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" round :loading="saving" @click="onCreate">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Grid, Plus, Refresh } from '@element-plus/icons-vue'
import { createApp, deleteApp, setDefaultApp } from '@/api/client'
import { useAppStore } from '@/stores/app'
import { formatDateTime } from '@/utils/formatDateTime'

const router = useRouter()
const route = useRoute()
const appStore = useAppStore()
const saving = ref(false)
const dialogVisible = ref(false)

const form = reactive({
  name: '',
  description: '',
})

async function loadApps() {
  try {
    await appStore.loadApps()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载失败')
  }
}

function openCreate() {
  form.name = ''
  form.description = ''
  dialogVisible.value = true
}

async function onCreate() {
  if (!form.name.trim()) {
    ElMessage.warning('请填写应用名称')
    return
  }
  saving.value = true
  try {
    const app = await createApp({
      name: form.name.trim(),
      description: form.description.trim() || undefined,
    })
    ElMessage.success('创建成功')
    dialogVisible.value = false
    await loadApps()
    appStore.rememberApp(app.id)
    router.push(`/apps/${app.id}/send`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '创建失败')
  } finally {
    saving.value = false
  }
}

async function onSetDefault(id: string) {
  try {
    await setDefaultApp(id)
    ElMessage.success('已设为默认应用')
    await loadApps()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '操作失败')
  }
}

async function onDelete(id: string) {
  try {
    await ElMessageBox.confirm('确定删除该应用吗？关联模板不会被自动删除。', '确认删除', {
      type: 'warning',
    })
    const wasCurrentRoute = route.params.id === id
    await deleteApp(id)
    ElMessage.success('删除成功')
    await loadApps()
    if (wasCurrentRoute) {
      router.push(appStore.resolveDefaultAppPath('send'))
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error instanceof Error ? error.message : '删除失败')
    }
  }
}

onMounted(async () => {
  await loadApps()
  if (appStore.apps.length === 0) {
    openCreate()
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

.app-table {
  border: none;
}

.app-table :deep(.el-table__inner-wrapper::before) {
  display: none;
}

.app-name-cell {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
}

.app-avatar {
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

.empty-apps {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-height: 280px;
  padding: 40px 24px;
  text-align: center;
}

.empty-apps h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 800;
  color: var(--ph-text);
}

.empty-apps p {
  margin: 0 0 8px;
  max-width: 420px;
  color: var(--ph-text-muted);
  line-height: 1.6;
}
</style>
