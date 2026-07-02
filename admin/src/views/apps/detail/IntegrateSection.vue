<script setup lang="ts">
import { toRef } from 'vue'
import CodeSnippet from '@/components/CodeSnippet.vue'
import type { AppConfig, AppInitSnippet, Template } from '@/api/types'
import { useAppIntegrateGuide } from '@/composables/useAppIntegrateGuide'

const props = defineProps<{
  appId: string
  app: AppConfig | null
  templates: Template[]
  initSnippet: AppInitSnippet | null
  loading: boolean
}>()

const emit = defineEmits<{
  copy: [text: string]
}>()

const {
  integrateTab,
  pushApiKey,
  configuredVendors,
  androidDownloadSnippet,
  androidDepsSnippet,
  androidManifestPlaceholdersSnippet,
  androidApplicationSnippet,
  androidServiceSnippet,
  androidManifestServiceSnippet,
  androidDeviceIdSnippet,
  backendFields,
  backendAuthHeaderSnippet,
  backendJsonSnippetHint,
  backendJsonSnippet,
  backendCurlSnippet,
} = useAppIntegrateGuide({
  appId: toRef(props, 'appId'),
  app: toRef(props, 'app'),
  templates: toRef(props, 'templates'),
  initSnippet: toRef(props, 'initSnippet'),
})

function onCopy(text?: string | null) {
  if (text) emit('copy', text)
}
</script>

<template>
  <section class="workspace-section">
    <header class="section-header">
      <h2 class="section-header__title">接入指南</h2>
      <p class="section-header__desc">
        下载 AAR → 配置 manifestPlaceholders → 初始化 SDK；厂商 SDK 已内嵌，无需 Maven 或 json 配置文件
      </p>
    </header>

    <div v-loading="loading" class="integrate-page">
      <div class="integrate-meta">
        <div class="integrate-meta__item">
          <span class="integrate-meta__label">应用 ID</span>
          <div class="integrate-meta__value">
            <code>{{ appId }}</code>
            <el-button link type="primary" @click="onCopy(appId)">复制</el-button>
          </div>
        </div>
        <div class="integrate-meta__item">
          <span class="integrate-meta__label">Push API Key</span>
          <div class="integrate-meta__value">
            <code>{{ pushApiKey || '加载中…' }}</code>
            <el-button v-if="pushApiKey" link type="primary" @click="onCopy(pushApiKey)">
              复制
            </el-button>
          </div>
        </div>
        <div class="integrate-meta__item integrate-meta__item--vendors">
          <span class="integrate-meta__label">已配置厂商</span>
          <div class="integrate-meta__tags">
            <el-tag
              v-for="item in configuredVendors"
              :key="item.id"
              size="small"
              effect="plain"
              type="success"
            >
              {{ item.label }}
            </el-tag>
            <span v-if="!configuredVendors.length" class="integrate-meta__empty">
              尚未配置，请先到「应用配置」填写
            </span>
          </div>
        </div>
      </div>

      <el-tabs v-model="integrateTab" class="integrate-tabs">
        <el-tab-pane label="Android 接入" name="android">
          <div class="integrate-flow">
            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">1</span>
                <div>
                  <h3>引入 AAR 依赖</h3>
                  <p>
                    从 GitHub Releases 下载 AAR 到 <code>app/libs/</code>，只引入已在「应用配置」开通的厂商模块；未引入的厂商设备自动走在线推送。
                  </p>
                </div>
              </header>
              <CodeSnippet title="下载" lang="Text" :code="androidDownloadSnippet" @copy="onCopy" />
              <CodeSnippet
                title="app/build.gradle.kts"
                lang="Kotlin"
                :code="androidDepsSnippet"
                @copy="onCopy"
              />
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">2</span>
                <div>
                  <h3>配置 Manifest 占位符</h3>
                  <p>
                    包含 Push Hub 与全部厂商占位符；已开通通道会填入真实参数，未使用的厂商可留空字符串（对应 AAR 不引入即可）。
                  </p>
                </div>
              </header>
              <CodeSnippet
                title="app/build.gradle.kts"
                lang="Kotlin"
                :code="androidManifestPlaceholdersSnippet"
                @copy="onCopy"
              />
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">3</span>
                <div>
                  <h3>初始化 SDK</h3>
                  <p>
                    在
                    <code>Application.onCreate()</code>
                    中调用 <code>PushHub.init</code>，仅需指定消息 Service。
                  </p>
                </div>
              </header>
              <CodeSnippet
                title="Application.kt"
                lang="Kotlin"
                :code="androidApplicationSnippet"
                @copy="onCopy"
              />
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">4</span>
                <div>
                  <h3>实现消息 Service</h3>
                  <p>统一接收在线 / 厂商通道消息，并在 Manifest 中注册。</p>
                </div>
              </header>
              <CodeSnippet
                title="PushMessageService"
                lang="Kotlin"
                :code="androidServiceSnippet"
                @copy="onCopy"
              />
              <CodeSnippet
                title="AndroidManifest.xml"
                lang="XML"
                :code="androidManifestServiceSnippet"
                @copy="onCopy"
              />
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">5</span>
                <div>
                  <h3>获取 device_id</h3>
                  <p>上报到业务后端，发送推送时填入 <code>targets.device_ids</code>。</p>
                </div>
              </header>
              <CodeSnippet
                title="Kotlin"
                lang="Kotlin"
                :code="androidDeviceIdSnippet"
                @copy="onCopy"
              />
            </article>
          </div>
        </el-tab-pane>

        <el-tab-pane label="业务后端接入" name="backend">
          <div class="integrate-flow">
            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">1</span>
                <div>
                  <h3>准备模板与设备</h3>
                  <p>
                    先创建推送模板；客户端注册成功后，在「设备列表」可看到
                    <code>device_id</code>。
                  </p>
                </div>
              </header>
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">2</span>
                <div>
                  <h3>鉴权</h3>
                  <p>
                    业务后端调用公开推送 API 须携带本应用的 Push API Key（与管理端登录 JWT 不同）。
                  </p>
                </div>
              </header>
              <CodeSnippet
                title="请求头"
                lang="HTTP"
                :code="backendAuthHeaderSnippet"
                @copy="onCopy"
              />
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">3</span>
                <div>
                  <h3>调用推送 API</h3>
                  <p>
                    向 Push Hub 服务端请求
                    <code>POST /api/v1/push</code>
                    （非管理端
                    <code>/api/v1/admin/</code>
                    ）。开发环境下若 Vite 已代理 <code>/api</code>，基址可与管理端同源。
                  </p>
                </div>
              </header>
              <CodeSnippet title="curl" lang="Shell" :code="backendCurlSnippet" @copy="onCopy" />
              <p class="integrate-note">{{ backendJsonSnippetHint }}</p>
              <CodeSnippet title="请求体" lang="JSON" :code="backendJsonSnippet" @copy="onCopy" />
            </article>

            <article class="integrate-card">
              <header class="integrate-card__head">
                <span class="integrate-card__step">4</span>
                <div>
                  <h3>常用字段</h3>
                  <p>
                    按模板类型选用 title/body 或 title_variables/body_variables；
                    完整说明见 <code>docs/server-api.md</code>。
                  </p>
                </div>
              </header>
              <div class="integrate-field-grid">
                <div v-for="field in backendFields" :key="field.name" class="integrate-field">
                  <div class="integrate-field__head">
                    <code>{{ field.name }}</code>
                    <span class="integrate-field__type">{{ field.type }}</span>
                    <span
                      v-if="field.required"
                      class="integrate-field__required"
                      :class="`integrate-field__required--${field.requiredLevel || 'optional'}`"
                    >
                      {{ field.required }}
                    </span>
                  </div>
                  <span>{{ field.desc }}</span>
                </div>
              </div>
            </article>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </section>
</template>

<style scoped>
.integrate-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.integrate-meta {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  padding: 14px;
  border-radius: 16px;
  background: var(--ph-gradient-soft);
  border: 1px solid var(--ph-border);
}

.integrate-meta__item {
  min-width: 0;
}

.integrate-meta__item--vendors {
  grid-column: 1 / -1;
}

.integrate-meta__label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 700;
  color: var(--ph-text-muted);
}

.integrate-meta__value {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.integrate-meta__value code,
.integrate-meta__empty,
.integrate-card__head code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.integrate-meta__value code {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 4px 8px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.8);
  color: var(--ph-primary-dark);
  font-size: 12px;
}

.integrate-meta__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.integrate-meta__empty {
  font-size: 13px;
  color: var(--ph-text-muted);
}

.integrate-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
}

.integrate-flow {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.integrate-card {
  padding: 16px;
  border-radius: 16px;
  background: var(--ph-surface);
  border: 1px solid var(--ph-border);
  box-shadow: var(--ph-shadow-sm);
}

.integrate-card__head {
  display: flex;
  gap: 12px;
  margin-bottom: 14px;
}

.integrate-card__step {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  background: var(--ph-gradient);
  color: #fff;
  font-size: 12px;
  font-weight: 800;
}

.integrate-card__head h3 {
  margin: 0 0 4px;
  font-size: 15px;
  font-weight: 800;
  color: var(--ph-text);
}

.integrate-card__head p {
  margin: 0;
  color: var(--ph-text-muted);
  line-height: 1.55;
  font-size: 13px;
}

.integrate-note {
  margin: 12px 0 0;
  padding: 10px 12px;
  border-radius: var(--ph-radius-sm);
  background: rgba(99, 102, 241, 0.06);
  color: var(--ph-text-muted);
  font-size: 13px;
  line-height: 1.6;
}

.integrate-field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.integrate-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  border-radius: 12px;
  background: rgba(99, 102, 241, 0.03);
  border: 1px solid rgba(99, 102, 241, 0.08);
  font-size: 13px;
  line-height: 1.55;
  color: var(--ph-text-muted);
}

.integrate-field__head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.integrate-field code {
  padding: 2px 6px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.85);
  color: var(--ph-primary-dark);
  font-size: 12px;
}

.integrate-field__type {
  font-size: 11px;
  font-weight: 600;
  color: var(--ph-text-muted);
  opacity: 0.85;
}

.integrate-field__required {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
}

.integrate-field__required--required {
  background: rgba(239, 68, 68, 0.12);
  color: #dc2626;
}

.integrate-field__required--recommended {
  background: rgba(245, 158, 11, 0.14);
  color: #d97706;
}

.integrate-field__required--conditional {
  background: rgba(99, 102, 241, 0.12);
  color: var(--ph-primary-dark);
}

.integrate-field__required--optional {
  background: rgba(148, 163, 184, 0.16);
  color: #64748b;
}

@media (max-width: 960px) {
  .integrate-meta,
  .integrate-field-grid {
    grid-template-columns: 1fr;
  }
}
</style>
