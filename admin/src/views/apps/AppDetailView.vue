<template>
  <div v-loading="loading" class="ph-page ph-page--fill app-detail">
    <div v-if="app" class="ph-panel workspace-panel ph-panel--fill">
      <section v-show="section === 'send'" class="workspace-section">
        <header class="section-header">
          <h2 class="section-header__title">发送测试</h2>
          <p class="section-header__desc">选择模板与目标设备，快速验证推送链路</p>
        </header>
        <el-form :model="pushForm" label-width="120px" class="push-form">
          <el-form-item label="推送类型">
            <div class="push-kind-picker">
              <button
                type="button"
                class="push-kind-card"
                :class="{ 'is-active': pushForm.pushKind === 'public' }"
                @click="pushForm.pushKind = 'public'"
              >
                <span class="push-kind-card__title">公信</span>
                <span class="push-kind-card__desc">选择公信模板，发送时再填写标题和内容</span>
              </button>
              <button
                type="button"
                class="push-kind-card"
                :class="{ 'is-active': pushForm.pushKind === 'private' }"
                @click="pushForm.pushKind = 'private'"
              >
                <span class="push-kind-card__title">私信</span>
                <span class="push-kind-card__desc">选择私信模板，按模板配置填写内容或变量</span>
              </button>
            </div>
          </el-form-item>
          <el-form-item label="模板" required>
            <el-select
              v-model="pushForm.template_id"
              :placeholder="pushForm.pushKind === 'public' ? '选择公信模板' : '选择私信模板'"
              style="width: 100%"
            >
              <el-option
                v-for="item in selectableTemplates"
                :key="item.id"
                :label="item.name"
                :value="item.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item
            v-if="showDirectTitleInput"
            label="标题"
            required
          >
            <el-input
              v-model="pushForm.title"
              :placeholder="titlePlaceholder"
            />
          </el-form-item>

          <el-form-item
            v-if="showDirectBodyInput"
            label="内容"
            required
          >
            <el-input
              v-model="pushForm.body"
              type="textarea"
              :rows="3"
              :placeholder="bodyPlaceholder"
            />
          </el-form-item>

          <el-form-item v-if="showTemplateCompose" label="消息内容" class="template-compose-item">
            <div class="template-compose">
              <div class="msg-preview">
                <div v-if="selectedTemplate?.title" class="msg-preview__block">
                  <div class="msg-preview__tag msg-preview__tag--title">标题</div>
                  <div class="msg-preview__line msg-preview__line--title">
                    <template
                      v-for="(seg, index) in titleSegments"
                      :key="`title-live-${index}-${seg.type}-${seg.value}`"
                    >
                      <span v-if="seg.type === 'text'" class="msg-preview__text">{{ seg.value }}</span>
                      <span v-else class="msg-preview__blank">
                        <input
                          v-model="pushForm.titleVariables[seg.value]"
                          class="msg-preview__input"
                          :placeholder="seg.value"
                          type="text"
                        />
                      </span>
                    </template>
                  </div>
                </div>

                <div v-if="selectedTemplate?.title && selectedTemplate?.body" class="msg-preview__divider" />

                <div v-if="selectedTemplate?.body" class="msg-preview__block">
                  <div class="msg-preview__tag msg-preview__tag--body">内容</div>
                  <div class="msg-preview__line msg-preview__line--body">
                    <template
                      v-for="(seg, index) in bodySegments"
                      :key="`body-live-${index}-${seg.type}-${seg.value}`"
                    >
                      <span v-if="seg.type === 'text'" class="msg-preview__text">{{ seg.value }}</span>
                      <span v-else class="msg-preview__blank">
                        <textarea
                          v-model="pushForm.bodyVariables[seg.value]"
                          class="msg-preview__input msg-preview__input--body"
                          :placeholder="seg.value"
                          rows="1"
                          @input="autoResizeTextarea($event)"
                        />
                      </span>
                    </template>
                  </div>
                </div>
              </div>
              <p v-if="showTitleVariableFields || showBodyVariableFields" class="template-compose__hint">
                在预览虚线框中填写各变量，标题与内容变量互不影响
              </p>
            </div>
          </el-form-item>

          <el-form-item label="点击行为">
            <el-select v-model="pushForm.click_type" style="width: 100%">
              <el-option label="打开应用" value="open_app" />
              <el-option label="打开页面" value="open_page" />
              <el-option label="打开网页" value="open_web" />
            </el-select>
          </el-form-item>
          <el-form-item v-if="pushForm.click_type === 'open_page'" label="Activity">
            <el-input
              v-model="pushForm.activity"
              placeholder="com.jiangker.push.sample.DemoTargetActivity（全类名）"
            />
            <p class="ph-field-hint">须填写全类名，与 Manifest 中 activity 声明一致</p>
          </el-form-item>
          <el-form-item v-if="pushForm.click_type === 'open_page'" label="页面参数">
            <div class="click-params">
              <div
                v-for="(row, index) in pushForm.click_params"
                :key="row.id"
                class="click-params__row"
              >
                <el-input v-model="row.key" placeholder="参数名" class="click-params__key" />
                <el-select v-model="row.valueType" class="click-params__type" @change="onClickParamTypeChange(row)">
                  <el-option label="文本" value="string" />
                  <el-option label="数字" value="number" />
                  <el-option label="开关" value="boolean" />
                </el-select>
                <el-switch
                  v-if="row.valueType === 'boolean'"
                  v-model="row.boolValue"
                  class="click-params__value click-params__value--bool"
                  inline-prompt
                  active-text="是"
                  inactive-text="否"
                />
                <el-input
                  v-else
                  v-model="row.value"
                  :placeholder="row.valueType === 'number' ? '如 1001' : '参数值'"
                  class="click-params__value"
                />
                <el-button link type="danger" class="click-params__remove" @click="removeClickParam(index)">
                  删除
                </el-button>
              </div>
              <el-button class="click-params__add" @click="addClickParam">添加参数</el-button>
              <p class="ph-field-hint">与本次消息内容对应，写入目标页 Intent extras</p>
            </div>
          </el-form-item>
          <el-form-item v-if="pushForm.click_type === 'open_web'" label="URL">
            <el-input v-model="pushForm.url" placeholder="https://example.com" />
          </el-form-item>
          <el-form-item label="目标设备" required>
            <el-select
              v-model="pushForm.device_ids"
              multiple
              filterable
              placeholder="选择设备"
              style="width: 100%"
            >
              <el-option
                v-for="item in devices"
                :key="item.id"
                :label="`${item.platform} - ${item.id.slice(0, 8)}...`"
                :value="item.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item label="通知 ID">
            <el-input
              v-model="pushForm.notify_id"
              placeholder="可选，如 1001"
              clearable
            />
            <p class="ph-field-hint">填写后相同 ID 的新消息会覆盖旧通知；不填则不传给服务端</p>
          </el-form-item>
          <el-form-item label="缓存截止">
            <div class="ph-cache-until-field">
              <el-checkbox v-model="pushForm.overrideCacheUntil">指定截止时间</el-checkbox>
              <el-date-picker
                v-model="pushForm.cacheUntil"
                type="datetime"
                placeholder="选择截止时间"
                :shortcuts="cacheUntilShortcuts"
                :disabled="!pushForm.overrideCacheUntil"
                style="width: 100%"
              />
              <div v-if="!pushForm.overrideCacheUntil" class="ph-field-hint">
                按模板默认 {{ effectiveCacheDays }} 天，预计有效至
                {{ formatCacheUntilLabel(effectiveCacheUntil) }}
              </div>
              <div v-else class="ph-field-hint">
                本次将缓存至 {{ formatCacheUntilLabel(pushForm.cacheUntil) }}
              </div>
            </div>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" round :loading="sending" @click="onSendPush">发送推送</el-button>
          </el-form-item>
        </el-form>
        <el-alert
          v-if="pushResult"
          :title="`发送完成：成功 ${pushResult.success} / 总计 ${pushResult.total}`"
          type="success"
          show-icon
          :closable="false"
          class="push-result-alert"
        />
        <div v-if="pushResult?.job_id" style="margin-top: 8px">
          <el-button link type="primary" @click="openJobDetail(pushResult!.job_id!)">
            查看推送链路
          </el-button>
        </div>
      </section>

      <section v-show="section === 'templates'" class="workspace-section">
        <header class="section-header">
          <h2 class="section-header__title">推送模板</h2>
          <p class="section-header__desc">配置公信 / 私信模板与厂商通道策略</p>
        </header>
        <div class="ph-toolbar">
          <el-button type="primary" round @click="openTemplateDialog()">新建模板</el-button>
        </div>
        <el-table :data="templates" stripe>
          <el-table-column prop="name" label="名称" min-width="120" />
          <el-table-column label="类型" width="120">
            <template #default="{ row }">
              <el-tag :type="row.kind === 'public' ? 'warning' : 'primary'" size="small">
                {{ templateKindLabel(row.kind) }}
              </el-tag>
              <el-tag
                v-if="row.kind === 'private'"
                size="small"
                type="info"
                style="margin-left: 4px"
              >
                {{ templateContentModeLabel(row) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="标题" min-width="160" show-overflow-tooltip>
            <template #default="{ row }">
              {{ formatTemplateTitlePreview(row) }}
            </template>
          </el-table-column>
          <el-table-column label="内容" min-width="160" show-overflow-tooltip>
            <template #default="{ row }">
              {{ formatTemplateBodyPreview(row) }}
            </template>
          </el-table-column>
          <el-table-column label="通道" min-width="160" show-overflow-tooltip>
            <template #default="{ row }">{{ formatTemplateChannels(row) }}</template>
          </el-table-column>
          <el-table-column label="缓存天数" min-width="100">
            <template #default="{ row }">{{ formatTemplateCache(row) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="160" fixed="right">
            <template #default="{ row }">
              <el-button link type="primary" @click="openTemplateDialog(row)">编辑</el-button>
              <el-button link type="danger" @click="onDeleteTemplate(row.id)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </section>

      <section v-show="section === 'channels'" class="workspace-section">
        <header class="section-header section-header--with-action">
          <h2 class="section-header__title">推送通道</h2>
          <el-button round @click="loadChannels">刷新</el-button>
        </header>

        <div class="vendor-switcher" role="tablist" aria-label="厂商">
          <button
            v-for="vendor in VENDOR_PLATFORMS"
            :key="vendor.value"
            type="button"
            role="tab"
            class="vendor-switcher__item"
            :class="{ 'is-active': activeChannelVendor === vendor.value }"
            :aria-selected="activeChannelVendor === vendor.value"
            @click="activeChannelVendor = vendor.value"
          >
            <span class="vendor-switcher__name">{{ vendor.label }}</span>
            <span class="vendor-switcher__count">{{ vendorChannelCount(vendor.value) }}</span>
          </button>
        </div>

        <div class="vendor-panel">
          <!-- 华为 / vivo：公信默认 / 私信多选 -->
          <div
            v-if="isPublicPrivatePlatform(activeChannelVendor)"
            class="vendor-channel-body"
          >
            <div class="vendor-channel-block">
              <div class="vendor-channel-block__head">
                <h3>公信</h3>
                <el-tag type="success" size="small" effect="plain">默认可用</el-tag>
              </div>
              <div class="vivo-category-grid">
                <div
                  v-for="item in publicCategoriesFor(asPublicPrivatePlatform(activeChannelVendor))"
                  :key="item.code"
                  class="vivo-category-card vivo-category-card--public"
                >
                  <div class="vivo-category-card__name">{{ item.name }}</div>
                  <div class="vivo-category-card__code">{{ item.code }}</div>
                  <div class="vivo-category-card__desc">{{ item.description }}</div>
                </div>
              </div>
            </div>

            <div class="vendor-channel-block">
              <div class="vendor-channel-block__head">
                <h3>私信</h3>
                <el-tag size="small" effect="plain">勾选启用</el-tag>
              </div>
              <el-checkbox-group
                v-model="privateCodesByPlatform[asPublicPrivatePlatform(activeChannelVendor)]"
                class="vivo-private-checks"
                :disabled="savingCategoryPrivate[asPublicPrivatePlatform(activeChannelVendor)]"
                @change="(codes: string[]) => onCategoryPrivateChange(asPublicPrivatePlatform(activeChannelVendor), codes)"
              >
                <el-checkbox
                  v-for="item in privateCategoriesFor(asPublicPrivatePlatform(activeChannelVendor))"
                  :key="item.code"
                  :value="item.code"
                  class="vivo-private-check"
                  border
                >
                  <span class="vivo-private-check__label">
                    <strong>{{ item.name }}</strong>
                    <span>{{ item.code }}</span>
                  </span>
                  <span class="vivo-private-check__desc">{{ item.description }}</span>
                </el-checkbox>
              </el-checkbox-group>
            </div>
          </div>

          <!-- 魅族 / OPPO：分类均默认可用 -->
          <div
            v-else-if="isAlwaysOnMsgTypePlatform(activeChannelVendor)"
            class="vendor-channel-body"
          >
            <template v-if="activeChannelVendor === 'oppo'">
              <div class="vendor-channel-block">
                <div class="vendor-channel-block__head">
                  <h3>内容与营销（公信）</h3>
                  <el-tag type="success" size="small" effect="plain">默认可用</el-tag>
                </div>
                <div class="vivo-category-grid">
                  <div
                    v-for="item in categoriesByLevel('oppo', 'public')"
                    :key="item.code"
                    class="vivo-category-card vivo-category-card--public"
                  >
                    <div class="vivo-category-card__name">{{ item.name }}</div>
                    <div class="vivo-category-card__code">{{ item.code }}</div>
                    <div class="vivo-category-card__desc">{{ item.description }}</div>
                  </div>
                </div>
              </div>
              <div class="vendor-channel-block">
                <div class="vendor-channel-block__head">
                  <h3>通讯与服务（私信）</h3>
                  <el-tag type="success" size="small" effect="plain">默认可用</el-tag>
                </div>
                <div class="vivo-category-grid">
                  <div
                    v-for="item in categoriesByLevel('oppo', 'private')"
                    :key="item.code"
                    class="vivo-category-card vivo-category-card--public"
                  >
                    <div class="vivo-category-card__name">{{ item.name }}</div>
                    <div class="vivo-category-card__code">{{ item.code }}</div>
                    <div class="vivo-category-card__desc">{{ item.description }}</div>
                  </div>
                </div>
              </div>
            </template>
            <div v-else class="vendor-channel-block">
              <div class="vendor-channel-block__head">
                <h3>消息分类</h3>
                <el-tag type="success" size="small" effect="plain">默认可用</el-tag>
              </div>
              <div class="vivo-category-grid">
                <div
                  v-for="item in categoriesForPlatform('meizu')"
                  :key="item.code"
                  class="vivo-category-card vivo-category-card--public"
                >
                  <div class="vivo-category-card__name">{{ item.name }}</div>
                  <div class="vivo-category-card__code">{{ item.code }}</div>
                  <div class="vivo-category-card__desc">{{ item.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 荣耀：无需通道配置 -->
          <div
            v-else-if="isNoChannelPlatform(activeChannelVendor)"
            class="vendor-channel-body"
          >
            <el-empty description="荣耀无需配置推送通道，在「应用配置」中填写推送密钥即可发送" />
          </div>

          <!-- 小米 -->
          <div v-else class="vendor-channel-body">
            <div class="ph-toolbar ph-toolbar--inner">
              <el-button
                type="primary"
                round
                @click="openChannelDialog(undefined, activeChannelVendor)"
              >
                新建通道
              </el-button>
            </div>
            <el-table
              :data="channelsByPlatform(activeChannelVendor)"
              stripe
              empty-text="暂无通道，点击上方新增"
            >
              <el-table-column prop="name" label="名称" min-width="140" />
              <el-table-column label="通道值" min-width="200" show-overflow-tooltip>
                <template #default="{ row }">
                  {{ formatChannelCodeLabel(row.platform, row.code) }}
                </template>
              </el-table-column>
              <el-table-column prop="description" label="说明" min-width="160" show-overflow-tooltip />
              <el-table-column label="默认" width="80">
                <template #default="{ row }">
                  <el-tag v-if="row.is_default" type="success" size="small">默认</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="160" fixed="right">
                <template #default="{ row }">
                  <el-button link type="primary" @click="openChannelDialog(row)">编辑</el-button>
                  <el-button link type="danger" @click="onDeleteChannel(row.id)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </section>

      <section v-show="section === 'devices'" class="workspace-section">
        <header class="section-header">
          <h2 class="section-header__title">设备列表</h2>
          <p class="section-header__desc">已注册的设备与 Push Token</p>
        </header>
        <el-table :data="devices" stripe>
          <el-table-column prop="id" label="Device ID" min-width="280" />
          <el-table-column prop="platform" label="平台" width="100" />
          <el-table-column prop="push_token" label="Push Token" min-width="220" show-overflow-tooltip />
          <el-table-column label="最近在线" min-width="170">
            <template #default="{ row }">{{ formatDateTime(row.last_online_at) }}</template>
          </el-table-column>
        </el-table>
      </section>

      <section v-show="section === 'stats'" class="workspace-section">
        <header class="section-header">
          <h2 class="section-header__title">推送统计</h2>
          <p class="section-header__desc">设备注册、在线情况与推送任务概览</p>
        </header>
        <div class="ph-toolbar">
          <span class="ph-muted">推送统计周期</span>
          <el-select v-model="statsDays" style="width: 120px" @change="loadStats">
            <el-option label="7 天" :value="7" />
            <el-option label="14 天" :value="14" />
            <el-option label="30 天" :value="30" />
          </el-select>
          <el-button round @click="loadStats">刷新</el-button>
        </div>

        <template v-if="stats">
          <h3 class="stats-group-title">设备概览</h3>
          <el-row :gutter="16" class="stats-cards">
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--sky">
                <div class="ph-stat-card__label">注册设备</div>
                <div class="ph-stat-card__value">{{ stats.devices.total }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--violet">
                <div class="ph-stat-card__label">近期在线</div>
                <div class="ph-stat-card__value">{{ stats.devices.recent_online }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--pink">
                <div class="ph-stat-card__label">周期内新增</div>
                <div class="ph-stat-card__value">{{ stats.devices.new_in_period }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--amber">
                <div class="ph-stat-card__label">推送模板</div>
                <div class="ph-stat-card__value">{{ stats.template_count }}</div>
              </div>
            </el-col>
          </el-row>

          <h3 class="stats-group-title">推送概览（近 {{ stats.days }} 天）</h3>
          <el-row :gutter="16" class="stats-cards">
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--violet">
                <div class="ph-stat-card__label">推送任务</div>
                <div class="ph-stat-card__value">{{ stats.total_jobs }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--pink">
                <div class="ph-stat-card__label">目标设备</div>
                <div class="ph-stat-card__value">{{ stats.total_targets }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--mint">
                <div class="ph-stat-card__label">成功</div>
                <div class="ph-stat-card__value">{{ stats.success_targets }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="ph-stat-card ph-stat-card--amber">
                <div class="ph-stat-card__label">成功率</div>
                <div class="ph-stat-card__value">{{ (stats.success_rate * 100).toFixed(1) }}%</div>
              </div>
            </el-col>
          </el-row>

          <el-row :gutter="16" class="stats-detail-row">
            <el-col :span="12">
              <el-card shadow="never" class="inner-card">
                <template #header>
                  <span class="inner-card__title">设备按平台</span>
                </template>
                <el-table :data="stats.devices.by_platform" size="small" empty-text="暂无注册设备">
                  <el-table-column label="平台">
                    <template #default="{ row }">{{ platformLabel(row.platform) }}</template>
                  </el-table-column>
                  <el-table-column prop="count" label="设备数" width="90" />
                </el-table>
              </el-card>
            </el-col>
            <el-col :span="12">
              <el-card shadow="never" class="inner-card">
                <template #header>
                  <span class="inner-card__title">推送按平台（近 {{ stats.days }} 天）</span>
                </template>
                <el-table :data="stats.push_by_platform" size="small" empty-text="暂无推送记录">
                  <el-table-column label="平台">
                    <template #default="{ row }">{{ platformLabel(row.platform) }}</template>
                  </el-table-column>
                  <el-table-column prop="success" label="成功" width="70" />
                  <el-table-column prop="failed" label="失败" width="70" />
                </el-table>
              </el-card>
            </el-col>
          </el-row>
          <el-row :gutter="16" class="stats-detail-row">
            <el-col :span="24">
              <el-card shadow="never" class="inner-card">
                <template #header>
                  <span class="inner-card__title">按日（近 {{ stats.days }} 天）</span>
                </template>
                <el-table :data="stats.daily" size="small" max-height="240" empty-text="暂无推送记录">
                  <el-table-column prop="date" label="日期" width="110" />
                  <el-table-column prop="jobs" label="任务" width="70" />
                  <el-table-column prop="success" label="成功" width="70" />
                  <el-table-column prop="failed" label="失败" width="70" />
                </el-table>
              </el-card>
            </el-col>
          </el-row>
        </template>
      </section>

      <section v-show="section === 'jobs'" class="workspace-section">
        <header class="section-header">
          <h2 class="section-header__title">推送记录</h2>
          <p class="section-header__desc">历史推送任务与完整链路追踪</p>
        </header>
        <div class="ph-toolbar">
          <el-button round @click="loadJobs">刷新</el-button>
        </div>
        <el-table :data="pushJobs" stripe @row-click="onJobRowClick">
          <el-table-column label="时间" min-width="170">
            <template #default="{ row }">{{ formatDateTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column prop="template_name" label="模板" min-width="120" />
          <el-table-column prop="title" label="标题" min-width="160" show-overflow-tooltip />
          <el-table-column label="结果" width="120">
            <template #default="{ row }">
              {{ row.success_count }}/{{ row.total_targets }}
            </template>
          </el-table-column>
          <el-table-column label="操作" width="100" fixed="right">
            <template #default="{ row }">
              <el-button link type="primary" @click.stop="openJobDetail(row.id)">链路</el-button>
            </template>
          </el-table-column>
        </el-table>
      </section>

      <section v-show="section === 'config'" class="workspace-section">
        <header class="section-header">
          <h2 class="section-header__title">应用配置</h2>
          <p class="section-header__desc">
            配置 Push Hub 服务端调用各厂商 API 所需的凭证
          </p>
        </header>
        <el-form :model="configForm" label-width="160px" class="config-form">
          <div class="config-block">
            <h3 class="config-block__title">基本信息</h3>
            <el-form-item label="应用 ID">
              <el-input :model-value="appId" readonly>
                <template #append>
                  <el-button @click="copyAppId">复制</el-button>
                </template>
              </el-input>
            </el-form-item>
            <el-form-item label="应用名称">
              <el-input v-model="configForm.name" />
            </el-form-item>
            <el-form-item label="描述">
              <el-input v-model="configForm.description" type="textarea" :rows="2" />
            </el-form-item>
          </div>

          <div class="config-block">
            <h3 class="config-block__title">平台标识</h3>
            <el-form-item label="Android 包名">
              <el-input
                v-model="configForm.package_name"
                placeholder="如 com.example.app，Android 推送与设备关联用"
              />
            </el-form-item>
            <el-form-item label="Push API Key">
              <el-input :model-value="configForm.push_api_key" readonly>
                <template #append>
                  <el-button @click="copyText(configForm.push_api_key)">复制</el-button>
                </template>
              </el-input>
              <p class="field-hint">业务后端调用 <code>POST /api/v1/push</code> 时在 Authorization 头携带</p>
            </el-form-item>
            <el-form-item label="iOS Bundle ID">
              <el-input
                v-model="configForm.ios_bundle_id"
                placeholder="如 com.example.app，后续 iOS 接入时使用"
              />
            </el-form-item>
            <el-form-item label="鸿蒙 Bundle Name">
              <el-input
                v-model="configForm.harmony_bundle_name"
                placeholder="如 com.example.app，后续鸿蒙接入时使用"
              />
            </el-form-item>
          </div>

          <div class="config-block">
            <div class="config-block__head config-block__head--vendor">
              <h3 class="config-block__title">厂商凭证</h3>
              <el-button
                round
                :loading="validatingCredentials"
                @click="validateCredentials()"
              >
                验证已填写凭证
              </el-button>
            </div>
            <div v-if="credentialValidations.length" class="credential-validations">
              <el-alert
                v-for="item in credentialValidations"
                :key="item.platform"
                :title="`${item.label}：${item.message}`"
                :type="credentialAlertType(item.status)"
                :closable="false"
                show-icon
              />
            </div>
            <el-collapse v-model="configVendorExpanded" class="vendor-collapse">
            <el-collapse-item name="xiaomi">
              <template #title>
                <span class="vendor-collapse-title">
                  小米推送
                  <el-tag
                    v-if="vendorValidationTag('xiaomi')"
                    size="small"
                    :type="vendorValidationTag('xiaomi')!"
                    effect="plain"
                  >
                    {{ vendorValidationLabel('xiaomi') }}
                  </el-tag>
                </span>
              </template>
              <el-form-item label="App Secret">
                <el-input v-model="configForm.xiaomi_app_secret" show-password />
              </el-form-item>
              <el-button link type="primary" @click.stop="validateCredentials('xiaomi')">
                验证小米凭证
              </el-button>
            </el-collapse-item>

            <el-collapse-item name="huawei">
              <template #title>
                <span class="vendor-collapse-title">
                  华为推送
                  <el-tag
                    v-if="vendorValidationTag('huawei')"
                    size="small"
                    :type="vendorValidationTag('huawei')!"
                    effect="plain"
                  >
                    {{ vendorValidationLabel('huawei') }}
                  </el-tag>
                </span>
              </template>
              <el-form-item label="App ID">
                <el-input
                  v-model="configForm.huawei_app_id"
                  placeholder="AppGallery Connect 应用 ID（如 108525411）"
                />
              </el-form-item>
              <el-form-item label="OAuth Client ID">
                <el-input
                  v-model="configForm.huawei_oauth_client_id"
                  placeholder="默认同 App ID；勿填 client.client_id"
                />
              </el-form-item>
              <el-form-item label="App Secret">
                <el-input
                  v-model="configForm.huawei_app_secret"
                  show-password
                  placeholder="AGC 应用密钥（服务端 OAuth 用）"
                />
              </el-form-item>
              <el-button link type="primary" @click.stop="validateCredentials('huawei')">
                验证华为凭证
              </el-button>
            </el-collapse-item>

            <el-collapse-item name="oppo">
              <template #title>
                <span class="vendor-collapse-title">
                  OPPO 推送
                  <el-tag
                    v-if="vendorValidationTag('oppo')"
                    size="small"
                    :type="vendorValidationTag('oppo')!"
                    effect="plain"
                  >
                    {{ vendorValidationLabel('oppo') }}
                  </el-tag>
                </span>
              </template>
              <el-form-item label="App Key">
                <el-input v-model="configForm.oppo_app_key" />
              </el-form-item>
              <el-form-item label="Master Secret">
                <el-input v-model="configForm.oppo_master_secret" show-password />
              </el-form-item>
              <el-button link type="primary" @click.stop="validateCredentials('oppo')">
                验证 OPPO 凭证
              </el-button>
            </el-collapse-item>

            <el-collapse-item name="vivo">
              <template #title>
                <span class="vendor-collapse-title">
                  vivo 推送
                  <el-tag
                    v-if="vendorValidationTag('vivo')"
                    size="small"
                    :type="vendorValidationTag('vivo')!"
                    effect="plain"
                  >
                    {{ vendorValidationLabel('vivo') }}
                  </el-tag>
                </span>
              </template>
              <el-form-item label="App ID">
                <el-input v-model="configForm.vivo_app_id" />
              </el-form-item>
              <el-form-item label="App Key">
                <el-input v-model="configForm.vivo_app_key" />
              </el-form-item>
              <el-form-item label="App Secret">
                <el-input v-model="configForm.vivo_app_secret" show-password />
              </el-form-item>
              <el-button link type="primary" @click.stop="validateCredentials('vivo')">
                验证 vivo 凭证
              </el-button>
            </el-collapse-item>

            <el-collapse-item name="honor">
              <template #title>
                <span class="vendor-collapse-title">
                  荣耀推送
                  <el-tag
                    v-if="vendorValidationTag('honor')"
                    size="small"
                    :type="vendorValidationTag('honor')!"
                    effect="plain"
                  >
                    {{ vendorValidationLabel('honor') }}
                  </el-tag>
                </span>
              </template>
              <el-form-item label="App ID">
                <el-input
                  v-model="configForm.honor_app_id"
                  placeholder="推送 AppId（= 客户端 HONOR_APP_ID）"
                />
              </el-form-item>
              <el-form-item label="Client ID" required>
                <el-input
                  v-model="configForm.honor_oauth_client_id"
                  placeholder="推送服务页「Client ID」，不是 App ID"
                />
              </el-form-item>
              <el-form-item label="Client Secret" required>
                <el-input
                  v-model="configForm.honor_app_secret"
                  show-password
                  placeholder="推送服务页「Client Secret」"
                />
              </el-form-item>
              <p class="field-hint">
                控制台「应用查看」页通常有四项（App ID、App Secret、Client ID、Client Secret）。
                服务端 OAuth 只需填 App ID + Client ID + Client Secret；<strong>Client Secret 不是 App Secret</strong>。
              </p>
              <el-button link type="primary" @click.stop="validateCredentials('honor')">
                验证荣耀凭证
              </el-button>
            </el-collapse-item>

            <el-collapse-item name="meizu">
              <template #title>
                <span class="vendor-collapse-title">
                  魅族推送
                  <el-tag
                    v-if="vendorValidationTag('meizu')"
                    size="small"
                    :type="vendorValidationTag('meizu')!"
                    effect="plain"
                  >
                    {{ vendorValidationLabel('meizu') }}
                  </el-tag>
                </span>
              </template>
              <el-form-item label="App ID">
                <el-input v-model="configForm.meizu_app_id" />
              </el-form-item>
              <el-form-item label="App Secret">
                <el-input v-model="configForm.meizu_app_secret" show-password />
              </el-form-item>
              <el-button link type="primary" @click.stop="validateCredentials('meizu')">
                验证魅族凭证
              </el-button>
            </el-collapse-item>
            </el-collapse>
          </div>

          <div class="config-block config-block--online">
            <h3 class="config-block__title">在线推送</h3>
            <el-alert type="info" :closable="false" show-icon>
              有厂商通道的设备：优先 WebSocket 在线推送，未送达立即降级厂商离线推送。在线消息缓存天数由模板配置，发送时展示并计算具体截止时间；点击行为按本次消息填写，不绑定模板。
            </el-alert>
          </div>

          <el-form-item class="config-form__actions" label-width="0">
            <el-button type="primary" round :loading="savingConfig" @click="saveConfig">保存配置</el-button>
          </el-form-item>
        </el-form>
      </section>

      <IntegrateSection
        v-show="section === 'integrate'"
        :app-id="appId"
        :app="app"
        :templates="templates"
        :init-snippet="initSnippet"
        :loading="loadingSnippet"
        @copy="(text) => copyText(text)"
      />
    </div>

    <el-drawer v-model="jobDrawerVisible" title="推送链路" size="640px">
      <template v-if="jobDetail">
        <el-descriptions :column="1" border size="small" style="margin-bottom: 16px">
          <el-descriptions-item label="任务 ID">{{ jobDetail.job.id }}</el-descriptions-item>
          <el-descriptions-item label="模板">{{ jobDetail.job.template_name }}</el-descriptions-item>
          <el-descriptions-item label="标题">{{ jobDetail.job.title }}</el-descriptions-item>
          <el-descriptions-item label="结果">
            成功 {{ jobDetail.job.success_count }} / 失败 {{ jobDetail.job.failed_count }}
          </el-descriptions-item>
        </el-descriptions>

        <h4>任务事件</h4>
        <el-timeline v-if="jobDetail.job_events.length" style="margin-bottom: 16px">
          <el-timeline-item
            v-for="event in jobDetail.job_events"
            :key="event.id"
            :timestamp="formatDateTime(event.created_at)"
            :type="event.status === 'ok' ? 'success' : event.status === 'failed' ? 'danger' : event.status === 'skipped' ? 'warning' : 'info'"
          >
            <strong>{{ stageLabel(event.stage) }}</strong>
            <div>{{ event.detail }}</div>
          </el-timeline-item>
        </el-timeline>

        <h4>单条消息链路</h4>
        <div
          v-for="message in jobDetail.messages"
          :key="message.target.id"
          class="job-message-trace"
        >
          <el-descriptions :column="2" border size="small" class="job-message-trace__head">
            <el-descriptions-item label="消息 ID">{{ message.target.id }}</el-descriptions-item>
            <el-descriptions-item label="平台">{{ message.target.platform }}</el-descriptions-item>
            <el-descriptions-item label="路由">{{ message.target.route_decision }}</el-descriptions-item>
            <el-descriptions-item label="状态">{{ message.target.final_status }}</el-descriptions-item>
            <el-descriptions-item label="通道">{{ message.target.final_channel || '—' }}</el-descriptions-item>
            <el-descriptions-item label="在线消息 ID">{{ message.target.outbox_id || '—' }}</el-descriptions-item>
            <el-descriptions-item label="厂商消息 ID" :span="2">
              {{ message.target.vendor_message_id || '—' }}
            </el-descriptions-item>
          </el-descriptions>

          <el-timeline v-if="message.events.length" class="job-message-trace__timeline">
            <el-timeline-item
              v-for="event in message.events"
              :key="event.id"
              :timestamp="formatDateTime(event.created_at)"
              :type="event.status === 'ok' ? 'success' : event.status === 'failed' ? 'danger' : event.status === 'skipped' ? 'warning' : 'info'"
            >
              <strong>{{ stageLabel(event.stage) }}</strong>
              <span v-if="event.platform"> · {{ event.platform }}</span>
              <div>{{ event.detail }}</div>
              <div v-if="eventVendorMessageId(event)" class="job-message-trace__meta">
                厂商消息 ID：{{ eventVendorMessageId(event) }}
              </div>
              <div v-else-if="eventError(event)" class="job-message-trace__meta job-message-trace__meta--error">
                {{ eventError(event) }}
              </div>
            </el-timeline-item>
          </el-timeline>

          <el-descriptions
            v-if="message.outbox"
            :column="2"
            border
            size="small"
            class="job-message-trace__outbox"
            title="在线 Outbox"
          >
            <el-descriptions-item label="Outbox ID">{{ message.outbox.id }}</el-descriptions-item>
            <el-descriptions-item label="送达">{{ formatDateTime(message.outbox.delivered_at) }}</el-descriptions-item>
            <el-descriptions-item label="降级时间">{{ formatDateTime(message.outbox.fallback_sent_at) }}</el-descriptions-item>
            <el-descriptions-item label="降级平台">{{ message.outbox.fallback_platform || '—' }}</el-descriptions-item>
          </el-descriptions>
        </div>
      </template>
    </el-drawer>

    <el-dialog v-model="templateDialogVisible" :title="editingTemplate ? '编辑模板' : '新建模板'" width="640px">
      <el-form :model="templateForm" label-width="120px">
        <el-form-item required>
          <template #label>
            <span class="form-label-with-help">
              类型
              <el-tooltip
                effect="light"
                placement="top"
                trigger="click"
                :width="280"
                content="公信：仅配置通道与行为，发送时再填标题内容。私信：发送时必须选择模板，可配置为发送时自由填写或预设文案拼接。"
              >
                <button type="button" class="field-help-btn" aria-label="查看类型说明">
                  <el-icon><QuestionFilled /></el-icon>
                </button>
              </el-tooltip>
            </span>
          </template>
          <el-radio-group v-model="templateForm.kind">
            <el-radio value="public">公信</el-radio>
            <el-radio value="private">私信</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="名称" required>
          <el-input v-model="templateForm.name" />
        </el-form-item>
        <template v-if="templateForm.kind === 'private'">
          <el-form-item required>
            <template #label>
              <span class="form-label-with-help">
                内容模式
                <el-tooltip
                  effect="light"
                  placement="top"
                  trigger="click"
                  :width="280"
                  :content="'自由填写：发送时再填标题与内容。拼接模式：在此预设标题、正文，支持 {{变量}} 占位符。'"
                >
                  <button type="button" class="field-help-btn" aria-label="查看内容模式说明">
                    <el-icon><QuestionFilled /></el-icon>
                  </button>
                </el-tooltip>
              </span>
            </template>
            <el-radio-group v-model="templateForm.content_mode">
              <el-radio value="free">自由填写</el-radio>
              <el-radio value="compose">拼接模式</el-radio>
            </el-radio-group>
          </el-form-item>
          <template v-if="templateForm.content_mode === 'compose'">
            <el-form-item label="标题" required>
              <el-input v-model="templateForm.title" :placeholder="'支持 {{变量}}'" />
            </el-form-item>
            <el-form-item label="内容" required>
              <el-input v-model="templateForm.body" type="textarea" :rows="3" :placeholder="'支持 {{变量}}'" />
            </el-form-item>
          </template>
        </template>
        <el-form-item label="小米通道">
          <el-select
            v-model="templateForm.xiaomi_channel_id"
            clearable
            placeholder="选择小米 NotificationChannel"
            style="width: 100%"
          >
            <el-option
              v-for="item in xiaomiChannels"
              :key="item.id"
              :label="channelOptionLabel(item)"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="华为分类">
          <el-select
            v-model="templateForm.huawei_channel_id"
            clearable
            placeholder="选择华为公信 / 私信分类"
            style="width: 100%"
          >
            <el-option
              v-for="item in huaweiChannels"
              :key="item.id"
              :label="channelOptionLabel(item)"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="OPPO 分类">
          <el-select
            v-model="templateForm.oppo_channel_id"
            clearable
            placeholder="可选，IM 无需对应模板"
            style="width: 100%"
            @change="onOppoCategoryChange"
          >
            <el-option
              v-for="item in oppoChannels"
              :key="item.id"
              :label="channelOptionLabel(item)"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item
          v-if="selectedOppoNeedsPrivateTemplate"
          label="对应 OPPO 模板"
          required
        >
          <el-input
            v-model="templateForm.oppo_private_template_id"
            placeholder="OPPO 控制台「私信模板」审核通过的模板 ID；变量名须与 OPPO 占位符一致"
          />
        </el-form-item>
        <el-form-item label="vivo 分类">
          <el-select
            v-model="templateForm.vivo_channel_id"
            clearable
            placeholder="选择 vivo 公信 / 私信分类"
            style="width: 100%"
          >
            <el-option
              v-for="item in vivoChannels"
              :key="item.id"
              :label="channelOptionLabel(item)"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="魅族">
          <el-select
            v-model="templateForm.meizu_channel_id"
            clearable
            placeholder="公信 / 私信"
            style="width: 100%"
          >
            <el-option
              v-for="item in meizuChannels"
              :key="item.id"
              :label="channelOptionLabel(item)"
              :value="item.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item>
          <template #label>
            <span class="form-label-with-help">
              缓存天数
              <el-tooltip
                effect="light"
                placement="top"
                trigger="click"
                :width="260"
                content="发送后在线消息默认保留天数。具体截止时间在发送时按发送时刻计算。"
              >
                <button type="button" class="field-help-btn" aria-label="查看缓存天数说明">
                  <el-icon><QuestionFilled /></el-icon>
                </button>
              </el-tooltip>
            </span>
          </template>
          <el-input-number
            v-model="templateForm.message_cache_days"
            :min="1"
            :max="365"
            controls-position="right"
          />
          <span class="ph-inline-unit">天</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="templateDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="savingTemplate" @click="saveTemplate">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="channelDialogVisible"
      :title="editingChannel ? `编辑${platformLabel(channelForm.platform)}通道` : `新建${platformLabel(channelForm.platform)}通道`"
      width="560px"
    >
      <el-form :model="channelForm" label-width="100px">
        <el-form-item :label="channelCodeFieldLabel" required>
          <el-input
            v-model="channelForm.code"
            :placeholder="channelCodePlaceholder"
          />
        </el-form-item>
        <el-form-item label="名称" required>
          <el-input v-model="channelForm.name" :placeholder="channelNamePlaceholder" />
        </el-form-item>
        <el-form-item label="说明">
          <el-input v-model="channelForm.description" type="textarea" :rows="2" />
        </el-form-item>
        <el-form-item label="设为默认">
          <el-switch v-model="channelForm.is_default" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="channelDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="savingChannel" @click="saveChannel">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { QuestionFilled } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import IntegrateSection from '@/views/apps/detail/IntegrateSection.vue'
import {
  createChannel,
  createTemplate,
  deleteChannel,
  deleteTemplate,
  fetchApp,
  fetchChannels,
  fetchDevices,
  fetchInitSnippet,
  fetchPushJobDetail,
  fetchPushJobs,
  fetchPushStats,
  fetchTemplates,
  sendPush,
  updateApp,
  updateChannel,
  updateTemplate,
  validateAppCredentials,
} from '@/api/client'
import type {
  AppConfig,
  AppInitSnippet,
  ClickAction,
  Device,
  PushChannel,
  PushJob,
  PushJobDetail,
  PushStatsOverview,
  SendPushResponse,
  Template,
  TemplateChannels,
  TemplateContentMode,
  VendorCredentialValidation,
} from '@/api/types'
import {
  cacheUntilFromDays,
  cacheUntilShortcuts,
  defaultCacheUntil,
  formatCacheUntil,
  formatCacheUntilLabel,
} from '@/utils/cacheUntil'
import { formatDateTime } from '@/utils/formatDateTime'
import {
  extractTemplateVariables,
  parseTemplateSegments,
} from '@/utils/templateVariables'
import {
  CATEGORY_MANAGED_PLATFORMS,
  PUBLIC_PRIVATE_PLATFORMS,
  VENDOR_PLATFORMS,
  categoriesByLevel,
  categoriesForPlatform,
  defaultCategoriesFor,
  findCategory,
  formatCategoryLabel,
  formatChannelCodeLabel,
  isAlwaysOnMsgTypePlatform,
  isCategoryManagedPlatform,
  isNoChannelPlatform,
  isPrivateCategory,
  isPublicCategory,
  isPublicPrivatePlatform,
  oppoRequiresPrivateTemplate,
  privateCategoriesFor,
  publicCategoriesFor,
  type CategoryManagedPlatform,
  type PublicPrivatePlatform,
  type VendorPlatform,
} from '@/utils/vendorChannels'

const route = useRoute()
const router = useRouter()
const appId = computed(() => route.params.id as string)

const section = computed(() =>
  typeof route.meta.section === 'string' ? route.meta.section : 'send',
)

const loading = ref(false)
const app = ref<AppConfig | null>(null)
const configVendorExpanded = ref<string[]>([])
const templates = ref<Template[]>([])
const devices = ref<Device[]>([])
const savingConfig = ref(false)
const validatingCredentials = ref(false)
const credentialValidations = ref<VendorCredentialValidation[]>([])
const savingTemplate = ref(false)
const sending = ref(false)
const pushResult = ref<SendPushResponse | null>(null)
const statsDays = ref(7)
const stats = ref<PushStatsOverview | null>(null)
const pushJobs = ref<PushJob[]>([])
const jobDrawerVisible = ref(false)
const jobDetail = ref<PushJobDetail | null>(null)
const pushChannels = ref<PushChannel[]>([])
const channelDialogVisible = ref(false)
const editingChannel = ref<PushChannel | null>(null)
const savingChannel = ref(false)
const savingCategoryPrivate = reactive<Record<PublicPrivatePlatform, boolean>>({
  vivo: false,
  huawei: false,
})
const activeChannelVendor = ref<VendorPlatform>('xiaomi')
const privateCodesByPlatform = reactive<Record<PublicPrivatePlatform, string[]>>({
  vivo: [],
  huawei: [],
})

const xiaomiChannels = computed(() => channelsByPlatform('xiaomi'))
const huaweiChannels = computed(() => categoryChannelsFor('huawei'))
const oppoChannels = computed(() => categoryChannelsFor('oppo'))
const vivoChannels = computed(() => categoryChannelsFor('vivo'))
const meizuChannels = computed(() => categoryChannelsFor('meizu'))

function channelsByPlatform(platform: VendorPlatform) {
  return pushChannels.value.filter((item) => item.platform === platform)
}

function categoryChannelsFor(platform: CategoryManagedPlatform) {
  return pushChannels.value.filter((item) => {
    if (item.platform !== platform) return false
    return isPublicCategory(platform, item.code) || isPrivateCategory(platform, item.code)
  })
}

function vendorChannelCount(platform: VendorPlatform) {
  if (isNoChannelPlatform(platform)) return '—'
  if (isAlwaysOnMsgTypePlatform(platform)) {
    return categoriesForPlatform(platform).length
  }
  if (isPublicPrivatePlatform(platform)) {
    return publicCategoriesFor(platform).length + privateCodesByPlatform[platform].length
  }
  return channelsByPlatform(platform).length
}

function asPublicPrivatePlatform(platform: VendorPlatform): PublicPrivatePlatform {
  if (!isPublicPrivatePlatform(platform)) {
    throw new Error(`platform ${platform} is not public/private managed`)
  }
  return platform
}

const configForm = reactive({
  name: '',
  package_name: '',
  push_api_key: '',
  ios_bundle_id: '',
  harmony_bundle_name: '',
  description: '',
  xiaomi_app_secret: '',
  huawei_app_id: '',
  huawei_oauth_client_id: '',
  huawei_app_secret: '',
  oppo_app_key: '',
  oppo_master_secret: '',
  vivo_app_id: '',
  vivo_app_key: '',
  vivo_app_secret: '',
  honor_app_id: '',
  honor_oauth_client_id: '',
  honor_app_secret: '',
  meizu_app_id: '',
  meizu_app_secret: '',
})

const initSnippet = ref<AppInitSnippet | null>(null)
const loadingSnippet = ref(false)

type ClickParamValueType = 'string' | 'number' | 'boolean'

interface ClickParamRow {
  id: number
  key: string
  valueType: ClickParamValueType
  value: string
  boolValue: boolean
}

let clickParamSeq = 0

function createClickParamRow(): ClickParamRow {
  clickParamSeq += 1
  return {
    id: clickParamSeq,
    key: '',
    valueType: 'string',
    value: '',
    boolValue: false,
  }
}

const pushForm = reactive({
  pushKind: 'public' as 'public' | 'private',
  template_id: '',
  title: '',
  body: '',
  titleVariables: {} as Record<string, string>,
  bodyVariables: {} as Record<string, string>,
  device_ids: [] as string[],
  click_type: 'open_app' as 'open_app' | 'open_page' | 'open_web',
  activity: '',
  click_params: [] as ClickParamRow[],
  url: '',
  notify_id: '',
  overrideCacheUntil: false,
  cacheUntil: defaultCacheUntil(7) as Date,
})

function isFullyQualifiedActivity(activity: string): boolean {
  const value = activity.trim()
  if (!value || value.startsWith('.') || value.includes('/')) return false
  const parts = value.split('.')
  if (parts.length < 2) return false
  return parts.every((part) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(part))
}

function addClickParam() {
  pushForm.click_params.push(createClickParamRow())
}

function removeClickParam(index: number) {
  pushForm.click_params.splice(index, 1)
}

function onClickParamTypeChange(row: ClickParamRow) {
  if (row.valueType === 'boolean') {
    const lowered = row.value.trim().toLowerCase()
    row.boolValue = lowered === 'true' || lowered === '1' || lowered === 'yes'
    return
  }
  if (row.valueType === 'number' && row.boolValue) {
    row.value = '1'
  } else if (row.valueType === 'string' && row.value === '' && row.boolValue) {
    row.value = 'true'
  }
}

function buildClickParams(): { params?: Record<string, unknown>; error?: string } {
  const params: Record<string, unknown> = {}
  const seen = new Set<string>()
  for (const row of pushForm.click_params) {
    const key = row.key.trim()
    if (!key && !row.value.trim() && row.valueType !== 'boolean') continue
    if (!key) return { error: '页面参数名不能为空' }
    if (seen.has(key)) return { error: `页面参数名重复：${key}` }
    seen.add(key)
    if (row.valueType === 'boolean') {
      params[key] = row.boolValue
      continue
    }
    const raw = row.value.trim()
    if (!raw) return { error: `请填写参数 ${key} 的值` }
    if (row.valueType === 'number') {
      const num = Number(raw)
      if (!Number.isFinite(num)) return { error: `参数 ${key} 须为有效数字` }
      params[key] = num
      continue
    }
    params[key] = row.value
  }
  return Object.keys(params).length ? { params } : {}
}

const templateDialogVisible = ref(false)
const editingTemplate = ref<Template | null>(null)
const templateForm = reactive({
  kind: 'private' as 'public' | 'private',
  content_mode: 'compose' as TemplateContentMode,
  name: '',
  title: '',
  body: '',
  xiaomi_channel_id: '',
  huawei_channel_id: '',
  oppo_channel_id: '',
  oppo_private_template_id: '',
  vivo_channel_id: '',
  meizu_channel_id: '',
  message_cache_days: 7,
})

const selectedOppoCategoryCode = computed(() => {
  const ch = pushChannels.value.find((item) => item.id === templateForm.oppo_channel_id)
  return ch?.code
})
const selectedOppoNeedsPrivateTemplate = computed(() =>
  oppoRequiresPrivateTemplate(selectedOppoCategoryCode.value),
)

const selectedTemplate = computed(() =>
  templates.value.find((item) => item.id === pushForm.template_id) || null,
)

const selectableTemplates = computed(() =>
  templates.value.filter((item) => item.kind === pushForm.pushKind),
)

function templateContentMode(template?: Template | null): TemplateContentMode {
  if (!template || template.kind === 'public') return 'compose'
  return template.content_mode ?? 'compose'
}

function isPrivateFreeTemplate(template?: Template | null) {
  return template?.kind === 'private' && templateContentMode(template) === 'free'
}

function isPrivateComposeTemplate(template?: Template | null) {
  return template?.kind === 'private' && templateContentMode(template) === 'compose'
}

const titlePlaceholder = computed(() => {
  if (pushForm.pushKind === 'public') return '公信推送标题'
  return '私信推送标题'
})

const bodyPlaceholder = computed(() => {
  if (pushForm.pushKind === 'public') return '公信推送内容'
  return '私信推送内容'
})

const titleTemplateVars = computed(() => {
  if (!isPrivateComposeTemplate(selectedTemplate.value)) return []
  return extractTemplateVariables(selectedTemplate.value!.title)
})

const bodyTemplateVars = computed(() => {
  if (!isPrivateComposeTemplate(selectedTemplate.value)) return []
  return extractTemplateVariables(selectedTemplate.value!.body)
})

const showTitleVariableFields = computed(
  () => isPrivateComposeTemplate(selectedTemplate.value) && titleTemplateVars.value.length > 0,
)

const showBodyVariableFields = computed(
  () => isPrivateComposeTemplate(selectedTemplate.value) && bodyTemplateVars.value.length > 0,
)

const showDirectTitleInput = computed(() => {
  if (pushForm.pushKind === 'public') return true
  if (!pushForm.template_id) return false
  return isPrivateFreeTemplate(selectedTemplate.value)
})

const showDirectBodyInput = computed(() => {
  if (pushForm.pushKind === 'public') return true
  if (!pushForm.template_id) return false
  return isPrivateFreeTemplate(selectedTemplate.value)
})

const showTemplateCompose = computed(
  () =>
    isPrivateComposeTemplate(selectedTemplate.value)
    && (showTitleVariableFields.value || showBodyVariableFields.value),
)

const titleSegments = computed(() => {
  if (!selectedTemplate.value) return []
  return parseTemplateSegments(selectedTemplate.value.title)
})

const bodySegments = computed(() => {
  if (!selectedTemplate.value) return []
  return parseTemplateSegments(selectedTemplate.value.body)
})

function autoResizeTextarea(event: Event) {
  const target = event.target as HTMLTextAreaElement
  target.style.height = 'auto'
  target.style.height = `${target.scrollHeight}px`
}

function syncTemplateVariableFields() {
  pushForm.titleVariables = Object.fromEntries(
    titleTemplateVars.value.map((name) => [name, pushForm.titleVariables[name] || '']),
  )
  pushForm.bodyVariables = Object.fromEntries(
    bodyTemplateVars.value.map((name) => [name, pushForm.bodyVariables[name] || '']),
  )
}

watch(
  () => templateForm.kind,
  (kind) => {
    if (kind === 'public') {
      templateForm.content_mode = 'compose'
      templateForm.title = ''
      templateForm.body = ''
    }
  },
)

watch(
  () => templateForm.content_mode,
  (mode) => {
    if (mode === 'free') {
      templateForm.title = ''
      templateForm.body = ''
    }
  },
)

watch(
  () => pushForm.pushKind,
  () => {
    pushForm.template_id = ''
    pushForm.title = ''
    pushForm.body = ''
    pushForm.titleVariables = {}
    pushForm.bodyVariables = {}
  },
)

watch(
  () => pushForm.template_id,
  (templateId) => {
    if (pushForm.overrideCacheUntil) return
    const template = templates.value.find((item) => item.id === templateId)
    if (template) {
      pushForm.cacheUntil = cacheUntilFromDays(template.message_cache_days ?? 7)
    }
    pushForm.title = ''
    pushForm.body = ''
    syncTemplateVariableFields()
  },
)

watch([titleTemplateVars, bodyTemplateVars], syncTemplateVariableFields)

function templateKindLabel(kind?: string) {
  return kind === 'public' ? '公信' : '私信'
}

function templateContentModeLabel(template: Template) {
  if (template.kind === 'public') return '-'
  return templateContentMode(template) === 'free' ? '自由填写' : '拼接'
}

function formatTemplateTitlePreview(template: Template) {
  if (template.kind === 'public') return '发送时填写'
  if (isPrivateFreeTemplate(template)) return '发送时填写'
  return template.title || '-'
}

function formatTemplateBodyPreview(template: Template) {
  if (template.kind === 'public') return '发送时填写'
  if (isPrivateFreeTemplate(template)) return '发送时填写'
  return template.body || '-'
}

const channelForm = reactive({
  platform: 'xiaomi' as VendorPlatform,
  name: '',
  code: '',
  description: '',
  is_default: false,
})

const channelCodeFieldLabel = computed(() => {
  switch (channelForm.platform) {
    case 'oppo':
      return '通道 ID'
    default:
      return '通道 ID'
  }
})

const channelCodePlaceholder = computed(() => {
  switch (channelForm.platform) {
    case 'oppo':
      return 'OPPO 开放平台申请的 Channel ID'
    default:
      return '小米 NotificationChannel ID'
  }
})

const channelNamePlaceholder = computed(() => {
  switch (channelForm.platform) {
    case 'oppo':
      return '如：默认通知'
    default:
      return '如：默认通知、订单消息'
  }
})

function platformLabel(platform: string) {
  if (platform === 'online') return '在线'
  if (platform === 'xiaomi') return '小米'
  if (platform === 'huawei') return '华为'
  if (platform === 'oppo') return 'OPPO'
  if (platform === 'vivo') return 'vivo'
  if (platform === 'honor') return '荣耀'
  if (platform === 'meizu') return '魅族'
  return platform
}

function channelOptionLabel(channel: PushChannel) {
  const suffix = channel.is_default ? ' · 默认' : ''
  if (isCategoryManagedPlatform(channel.platform)) {
    return `${formatChannelCodeLabel(channel.platform, channel.code, channel.name)}${suffix}`
  }
  return `${channel.name} (${channel.code})${suffix}`
}

function findChannelByCode(platform: string, code?: string) {
  if (!code) return ''
  return pushChannels.value.find((item) => item.platform === platform && item.code === code)?.id || ''
}

function findCategoryChannelId(platform: CategoryManagedPlatform, category?: string) {
  if (!category) return ''
  const normalized = category.toUpperCase()
  return (
    pushChannels.value.find(
      (item) => item.platform === platform && item.code.toUpperCase() === normalized,
    )?.id || ''
  )
}

function formatTemplateChannels(template: Template) {
  const parts: string[] = []
  const xiaomiCode = template.channels?.xiaomi?.channel_id
  if (xiaomiCode) {
    const ch = pushChannels.value.find(
      (item) => item.platform === 'xiaomi' && item.code === xiaomiCode,
    )
    parts.push(`小米: ${ch?.name || xiaomiCode}`)
  }
  const huaweiCategory = template.channels?.huawei?.category
  if (huaweiCategory) {
    const preset = findCategory('huawei', huaweiCategory)
    parts.push(`华为: ${preset ? formatCategoryLabel(preset) : huaweiCategory}`)
  }
  const oppoTpl = template.channels?.oppo?.private_template_id
  const oppoCategory = template.channels?.oppo?.category
    || template.channels?.oppo?.channel_id
  if (oppoTpl) {
    parts.push('OPPO: 对应模板')
  } else if (oppoCategory) {
    const preset = findCategory('oppo', oppoCategory)
    parts.push(`OPPO: ${preset ? formatCategoryLabel(preset) : oppoCategory}`)
  }
  const vivoCategory = template.channels?.vivo?.category
  if (vivoCategory) {
    const preset = findCategory('vivo', vivoCategory)
    parts.push(`vivo: ${preset ? formatCategoryLabel(preset) : vivoCategory}`)
  }
  const meizuMsgType = template.channels?.meizu?.msg_type
  if (meizuMsgType) {
    const preset = findCategory('meizu', meizuMsgType)
    parts.push(`魅族: ${preset ? formatCategoryLabel(preset) : meizuMsgType}`)
  }
  return parts.join(' / ') || '-'
}

function formatTemplateCache(template: Template) {
  return `${template.message_cache_days ?? 7} 天`
}

const effectiveCacheDays = computed(() => {
  if (selectedTemplate.value) return selectedTemplate.value.message_cache_days ?? 7
  return 7
})

const effectiveCacheUntil = computed(() => cacheUntilFromDays(effectiveCacheDays.value))

function syncConfigVendorExpanded(data: AppConfig) {
  const expanded: string[] = []
  if (data.xiaomi_app_secret) expanded.push('xiaomi')
  if (data.huawei_app_id || data.huawei_app_secret) expanded.push('huawei')
  if (data.oppo_app_key || data.oppo_master_secret) expanded.push('oppo')
  if (data.vivo_app_id || data.vivo_app_key || data.vivo_app_secret) expanded.push('vivo')
  if (data.honor_app_id || data.honor_oauth_client_id || data.honor_app_secret) expanded.push('honor')
  if (data.meizu_app_id || data.meizu_app_secret) expanded.push('meizu')
  configVendorExpanded.value = expanded
}

function fillConfigForm(data: AppConfig) {
  configForm.name = data.name
  configForm.package_name = data.package_name
  configForm.push_api_key = data.push_api_key || ''
  configForm.ios_bundle_id = data.ios_bundle_id || ''
  configForm.harmony_bundle_name = data.harmony_bundle_name || ''
  configForm.description = data.description || ''
  configForm.xiaomi_app_secret = data.xiaomi_app_secret || ''
  configForm.huawei_app_id = data.huawei_app_id || ''
  configForm.huawei_oauth_client_id = data.huawei_oauth_client_id || ''
  configForm.huawei_app_secret = data.huawei_app_secret || ''
  configForm.oppo_app_key = data.oppo_app_key || ''
  configForm.oppo_master_secret = data.oppo_master_secret || ''
  configForm.vivo_app_id = data.vivo_app_id || ''
  configForm.vivo_app_key = data.vivo_app_key || ''
  configForm.vivo_app_secret = data.vivo_app_secret || ''
  configForm.honor_app_id = data.honor_app_id || ''
  configForm.honor_oauth_client_id = data.honor_oauth_client_id || ''
  configForm.honor_app_secret = data.honor_app_secret || ''
  configForm.meizu_app_id = data.meizu_app_id || ''
  configForm.meizu_app_secret = data.meizu_app_secret || ''
  syncConfigVendorExpanded(data)
}

const stageLabels: Record<string, string> = {
  received: '收到请求',
  template_rendered: '模板渲染',
  online_cache: '在线缓存',
  route_selected: '路由决策',
  online_enqueue: '在线入队',
  online_ws: 'WebSocket 下发',
  online_ack: '客户端 ACK',
  client_display: '通知展示',
  vendor_send: '厂商推送',
  vendor_fallback: '厂商降级',
}

function stageLabel(stage: string) {
  return stageLabels[stage] || stage
}

function eventVendorMessageId(event: { metadata?: string }) {
  if (!event.metadata) return ''
  try {
    const data = JSON.parse(event.metadata) as { vendor_message_id?: string; message_id?: string }
    return data.vendor_message_id || data.message_id || ''
  } catch {
    return ''
  }
}

function eventError(event: { metadata?: string; status: string }) {
  if (event.status !== 'failed' || !event.metadata) return ''
  try {
    const data = JSON.parse(event.metadata) as { error?: string }
    return data.error || ''
  } catch {
    return ''
  }
}

function syncPrivateCodesFromChannels() {
  for (const platform of PUBLIC_PRIVATE_PLATFORMS) {
    privateCodesByPlatform[platform] = pushChannels.value
      .filter((item) => item.platform === platform && isPrivateCategory(platform, item.code))
      .map((item) => item.code.toUpperCase())
  }
}

/** 默认可用分类写入通道表：vivo/华为仅公信，魅族/OPPO 全部 */
async function ensurePublicChannels() {
  let created = false
  for (const platform of CATEGORY_MANAGED_PLATFORMS) {
    const existing = new Set(
      pushChannels.value
        .filter((item) => item.platform === platform)
        .map((item) => item.code.toUpperCase()),
    )
    for (const preset of defaultCategoriesFor(platform)) {
      if (existing.has(preset.code)) continue
      await createChannel(appId.value, {
        platform,
        name: preset.name,
        code: preset.code,
        description: preset.description,
        is_default: false,
      })
      created = true
    }
  }
  if (created) {
    pushChannels.value = await fetchChannels(appId.value)
  }
}

async function loadChannels() {
  try {
    pushChannels.value = await fetchChannels(appId.value)
    await ensurePublicChannels()
    syncPrivateCodesFromChannels()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载通道失败')
  }
}

async function onCategoryPrivateChange(
  platform: PublicPrivatePlatform,
  codes: string[] | unknown,
) {
  const wanted = new Set(
    (Array.isArray(codes) ? codes : privateCodesByPlatform[platform]).map((code) =>
      String(code).toUpperCase(),
    ),
  )
  savingCategoryPrivate[platform] = true
  try {
    const existing = pushChannels.value.filter(
      (item) => item.platform === platform && isPrivateCategory(platform, item.code),
    )
    for (const channel of existing) {
      if (!wanted.has(channel.code.toUpperCase())) {
        await deleteChannel(channel.id)
      }
    }
    const existingCodes = new Set(existing.map((item) => item.code.toUpperCase()))
    for (const code of wanted) {
      if (existingCodes.has(code)) continue
      const preset = findCategory(platform, code)
      if (!preset || preset.level !== 'private') continue
      await createChannel(appId.value, {
        platform,
        name: preset.name,
        code: preset.code,
        description: preset.description,
        is_default: false,
      })
    }
    pushChannels.value = await fetchChannels(appId.value)
    await ensurePublicChannels()
    syncPrivateCodesFromChannels()
  } catch (error) {
    syncPrivateCodesFromChannels()
    ElMessage.error(
      error instanceof Error ? error.message : `更新 ${platformLabel(platform)} 私信通道失败`,
    )
  } finally {
    savingCategoryPrivate[platform] = false
  }
}

function openChannelDialog(channel?: PushChannel, platform?: VendorPlatform) {
  const targetPlatform = (channel?.platform as VendorPlatform) || platform || 'xiaomi'
  if (isCategoryManagedPlatform(targetPlatform)) return
  editingChannel.value = channel || null
  channelForm.platform = targetPlatform
  channelForm.name = channel?.name || ''
  channelForm.code = channel?.code || ''
  channelForm.description = channel?.description || ''
  channelForm.is_default = channel?.is_default || false
  channelDialogVisible.value = true
}

async function saveChannel() {
  if (isCategoryManagedPlatform(channelForm.platform)) return
  if (!channelForm.code.trim()) {
    ElMessage.warning('请填写通道值')
    return
  }
  if (!channelForm.name.trim()) {
    ElMessage.warning('请填写名称')
    return
  }
  savingChannel.value = true
  try {
    const payload = {
      platform: channelForm.platform,
      name: channelForm.name.trim(),
      code: channelForm.code.trim(),
      description: channelForm.description.trim() || undefined,
      is_default: channelForm.is_default,
    }
    if (editingChannel.value) {
      await updateChannel(editingChannel.value.id, payload)
    } else {
      await createChannel(appId.value, payload)
    }
    channelDialogVisible.value = false
    await loadChannels()
    ElMessage.success('通道已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存失败')
  } finally {
    savingChannel.value = false
  }
}

async function onDeleteChannel(id: string) {
  const target = pushChannels.value.find((item) => item.id === id)
  if (
    target
    && isCategoryManagedPlatform(target.platform)
    && isPublicCategory(target.platform, target.code)
  ) {
    ElMessage.warning(
      isAlwaysOnMsgTypePlatform(target.platform)
        ? `${platformLabel(target.platform)} 分类默认可用，不可删除`
        : `${platformLabel(target.platform)} 公信通道默认可用，不可删除`,
    )
    return
  }
  try {
    await ElMessageBox.confirm('确定删除该通道吗？', '确认删除', { type: 'warning' })
    await deleteChannel(id)
    await loadChannels()
    ElMessage.success('删除成功')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error instanceof Error ? error.message : '删除失败')
    }
  }
}

async function loadStats() {
  try {
    stats.value = await fetchPushStats(appId.value, statsDays.value)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载统计失败')
  }
}

async function loadJobs() {
  try {
    pushJobs.value = await fetchPushJobs(appId.value)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载推送记录失败')
  }
}

async function openJobDetail(jobId: string) {
  try {
    jobDetail.value = await fetchPushJobDetail(appId.value, jobId)
    jobDrawerVisible.value = true
    if (section.value !== 'jobs') {
      router.push(`/apps/${appId.value}/jobs`)
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载链路失败')
  }
}

function onJobRowClick(row: PushJob) {
  openJobDetail(row.id)
}

async function loadAll() {
  loading.value = true
  try {
    const [appData, templateData, deviceData] = await Promise.all([
      fetchApp(appId.value),
      fetchTemplates(appId.value),
      fetchDevices(appId.value),
    ])
    app.value = appData
    fillConfigForm(appData)
    templates.value = templateData
    devices.value = deviceData
    await Promise.all([loadStats(), loadJobs(), loadChannels(), loadInitSnippet()])
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载失败')
  } finally {
    loading.value = false
  }
}

async function loadInitSnippet() {
  loadingSnippet.value = true
  try {
    initSnippet.value = await fetchInitSnippet(appId.value)
  } catch (error) {
    initSnippet.value = null
    if (section.value === 'integrate') {
      ElMessage.error(error instanceof Error ? error.message : '加载接入代码失败')
    }
  } finally {
    loadingSnippet.value = false
  }
}

async function copyAppId() {
  await copyText(appId.value, '已复制应用 ID')
}

async function copyText(text?: string | null, successMessage = '已复制') {
  if (!text) {
    ElMessage.warning('暂无可复制内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(successMessage)
  } catch {
    ElMessage.error('复制失败')
  }
}

function buildValidatePayload(platform?: string) {
  return {
    platform,
    package_name: configForm.package_name.trim() || undefined,
    xiaomi_app_secret: configForm.xiaomi_app_secret.trim() || undefined,
    huawei_app_id: configForm.huawei_app_id.trim() || undefined,
    huawei_oauth_client_id: configForm.huawei_oauth_client_id.trim() || undefined,
    huawei_app_secret: configForm.huawei_app_secret.trim() || undefined,
    oppo_app_key: configForm.oppo_app_key.trim() || undefined,
    oppo_master_secret: configForm.oppo_master_secret.trim() || undefined,
    vivo_app_id: configForm.vivo_app_id.trim() || undefined,
    vivo_app_key: configForm.vivo_app_key.trim() || undefined,
    vivo_app_secret: configForm.vivo_app_secret.trim() || undefined,
    honor_app_id: configForm.honor_app_id.trim() || undefined,
    honor_oauth_client_id: configForm.honor_oauth_client_id.trim() || undefined,
    honor_app_secret: configForm.honor_app_secret.trim() || undefined,
    meizu_app_id: configForm.meizu_app_id.trim() || undefined,
    meizu_app_secret: configForm.meizu_app_secret.trim() || undefined,
  }
}

function mergeCredentialValidations(
  results: VendorCredentialValidation[],
  platform?: string,
) {
  if (!platform) {
    credentialValidations.value = results
    return
  }
  const others = credentialValidations.value.filter((item) => item.platform !== platform)
  credentialValidations.value = [...others, ...results]
}

function credentialAlertType(status: VendorCredentialValidation['status']) {
  if (status === 'ok') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'incomplete') return 'warning'
  return 'info'
}

function vendorValidationTag(platform: string) {
  const item = credentialValidations.value.find((entry) => entry.platform === platform)
  if (!item || item.status === 'skipped') return ''
  if (item.status === 'ok') return 'success'
  if (item.status === 'failed') return 'danger'
  return 'warning'
}

function vendorValidationLabel(platform: string) {
  const item = credentialValidations.value.find((entry) => entry.platform === platform)
  if (!item) return ''
  if (item.status === 'ok') return '有效'
  if (item.status === 'failed') return '无效'
  if (item.status === 'incomplete') return '未填全'
  return '未配置'
}

async function validateCredentials(platform?: string) {
  validatingCredentials.value = true
  try {
    const results = await validateAppCredentials(appId.value, buildValidatePayload(platform))
    mergeCredentialValidations(results, platform)
    const checked = results.filter((item) => item.status !== 'skipped')
    if (!checked.length) {
      ElMessage.info(platform ? '当前厂商未填写凭证' : '没有需要验证的厂商凭证')
      return
    }
    const failed = checked.filter((item) => item.status === 'failed')
    if (failed.length) {
      ElMessage.error(failed.map((item) => `${item.label}：${item.message}`).join('；'))
      return
    }
    const incomplete = checked.filter((item) => item.status === 'incomplete')
    if (incomplete.length) {
      ElMessage.warning(incomplete.map((item) => `${item.label}：${item.message}`).join('；'))
      return
    }
    ElMessage.success(
      platform
        ? `${checked[0]?.label || '厂商'}凭证验证通过`
        : `已验证 ${checked.length} 个厂商，凭证均有效`,
    )
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '验证失败')
  } finally {
    validatingCredentials.value = false
  }
}

async function saveConfig() {
  savingConfig.value = true
  try {
    const data = await updateApp(appId.value, {
      name: configForm.name.trim(),
      package_name: configForm.package_name.trim() || undefined,
      server_base_url: app.value?.server_base_url || undefined,
      ios_bundle_id: configForm.ios_bundle_id.trim() || undefined,
      harmony_bundle_name: configForm.harmony_bundle_name.trim() || undefined,
      description: configForm.description.trim() || undefined,
      xiaomi_app_secret: configForm.xiaomi_app_secret.trim() || undefined,
      huawei_app_id: configForm.huawei_app_id.trim() || undefined,
      huawei_oauth_client_id: configForm.huawei_oauth_client_id.trim() || undefined,
      huawei_app_secret: configForm.huawei_app_secret.trim() || undefined,
      oppo_app_key: configForm.oppo_app_key.trim() || undefined,
      oppo_master_secret: configForm.oppo_master_secret.trim() || undefined,
      vivo_app_id: configForm.vivo_app_id.trim() || undefined,
      vivo_app_key: configForm.vivo_app_key.trim() || undefined,
      vivo_app_secret: configForm.vivo_app_secret.trim() || undefined,
      honor_app_id: configForm.honor_app_id.trim() || undefined,
      honor_oauth_client_id: configForm.honor_oauth_client_id.trim() || undefined,
      honor_app_secret: configForm.honor_app_secret.trim() || undefined,
      meizu_app_id: configForm.meizu_app_id.trim() || undefined,
      meizu_app_secret: configForm.meizu_app_secret.trim() || undefined,
    })
    app.value = data
    fillConfigForm(data)
    ElMessage.success('配置已保存')
    await Promise.all([validateCredentials(), loadInitSnippet()])
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存失败')
  } finally {
    savingConfig.value = false
  }
}

/** 选了非 IM 私信分类时，自动切到拼接模式私信，与 OPPO 审核模板对应。 */
function onOppoCategoryChange() {
  if (!selectedOppoNeedsPrivateTemplate.value) return
  templateForm.kind = 'private'
  templateForm.content_mode = 'compose'
}

function openTemplateDialog(template?: Template) {
  editingTemplate.value = template || null
  templateForm.kind = template?.kind || 'private'
  templateForm.content_mode = templateContentMode(template)
  templateForm.name = template?.name || ''
  templateForm.title = template?.title || ''
  templateForm.body = template?.body || ''
  templateForm.xiaomi_channel_id = findChannelByCode(
    'xiaomi',
    template?.channels?.xiaomi?.channel_id,
  )
  templateForm.huawei_channel_id = findCategoryChannelId(
    'huawei',
    template?.channels?.huawei?.category,
  )
  templateForm.oppo_channel_id = findCategoryChannelId(
    'oppo',
    template?.channels?.oppo?.category || template?.channels?.oppo?.channel_id,
  )
  templateForm.oppo_private_template_id = template?.channels?.oppo?.private_template_id || ''
  templateForm.vivo_channel_id = findCategoryChannelId('vivo', template?.channels?.vivo?.category)
  templateForm.meizu_channel_id = findCategoryChannelId(
    'meizu',
    template?.channels?.meizu?.msg_type,
  )
  templateForm.message_cache_days = template?.message_cache_days ?? 7
  templateDialogVisible.value = true
}

function buildTemplatePayload() {
  const channels: TemplateChannels = {}
  if (templateForm.xiaomi_channel_id) {
    const ch = pushChannels.value.find((item) => item.id === templateForm.xiaomi_channel_id)
    if (ch) channels.xiaomi = { channel_id: ch.code }
  }
  if (templateForm.huawei_channel_id) {
    const ch = pushChannels.value.find((item) => item.id === templateForm.huawei_channel_id)
    if (ch?.code) channels.huawei = { category: ch.code.trim().toUpperCase() }
  }
  {
    const ch = templateForm.oppo_channel_id
      ? pushChannels.value.find((item) => item.id === templateForm.oppo_channel_id)
      : undefined
    const category = (ch?.code || editingTemplate.value?.channels?.oppo?.category || '')
      .trim()
      .toUpperCase()
    const privateTemplateId = templateForm.oppo_private_template_id.trim()
    if (category || privateTemplateId) {
      channels.oppo = {}
      if (category) channels.oppo.category = category
      if (privateTemplateId && (!category || oppoRequiresPrivateTemplate(category))) {
        channels.oppo.private_template_id = privateTemplateId
      }
    }
  }
  if (templateForm.vivo_channel_id) {
    const ch = pushChannels.value.find((item) => item.id === templateForm.vivo_channel_id)
    if (ch?.code) channels.vivo = { category: ch.code.trim().toUpperCase() }
  }
  if (templateForm.meizu_channel_id) {
    const ch = pushChannels.value.find((item) => item.id === templateForm.meizu_channel_id)
    if (ch?.code) channels.meizu = { msg_type: ch.code.trim().toUpperCase() }
  }
  return {
    name: templateForm.name.trim(),
    kind: templateForm.kind,
    content_mode: templateForm.kind === 'private' ? templateForm.content_mode : 'compose',
    title: templateForm.kind === 'private' && templateForm.content_mode === 'compose'
      ? templateForm.title.trim()
      : '',
    body: templateForm.kind === 'private' && templateForm.content_mode === 'compose'
      ? templateForm.body.trim()
      : '',
    channels,
    message_cache_days: Math.max(1, Math.floor(templateForm.message_cache_days || 7)),
  }
}

async function saveTemplate() {
  if (!templateForm.name.trim()) {
    ElMessage.warning('请填写模板名称')
    return
  }
  if (
    templateForm.kind === 'private'
    && templateForm.content_mode === 'compose'
    && (!templateForm.title.trim() || !templateForm.body.trim())
  ) {
    ElMessage.warning('拼接模式私信模板请填写标题和内容')
    return
  }
  if (selectedOppoNeedsPrivateTemplate.value) {
    if (!templateForm.oppo_private_template_id.trim()) {
      ElMessage.warning('请填写对应的 OPPO 模板 ID')
      return
    }
    if (templateForm.kind !== 'private' || templateForm.content_mode !== 'compose') {
      ElMessage.warning('对应 OPPO 模板时须用拼接模式私信：发送填变量，其它平台自动拼正文')
      return
    }
  }
  if (!Number.isFinite(templateForm.message_cache_days) || templateForm.message_cache_days < 1) {
    ElMessage.warning('缓存天数须为不小于 1 的整数')
    return
  }
  savingTemplate.value = true
  try {
    const payload = buildTemplatePayload()
    if (editingTemplate.value) {
      await updateTemplate(editingTemplate.value.id, payload)
    } else {
      await createTemplate(appId.value, payload)
    }
    templateDialogVisible.value = false
    templates.value = await fetchTemplates(appId.value)
    ElMessage.success('模板已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存失败')
  } finally {
    savingTemplate.value = false
  }
}

async function onDeleteTemplate(id: string) {
  try {
    await ElMessageBox.confirm('确定删除该模板吗？', '确认删除', { type: 'warning' })
    await deleteTemplate(id)
    templates.value = await fetchTemplates(appId.value)
    ElMessage.success('删除成功')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error instanceof Error ? error.message : '删除失败')
    }
  }
}

async function onSendPush() {
  if (pushForm.device_ids.length === 0) {
    ElMessage.warning('请选择目标设备')
    return
  }
  const hasTemplate = !!pushForm.template_id
  const pushTitle = pushForm.title.trim()
  const pushBody = pushForm.body.trim()

  if (!hasTemplate) {
    ElMessage.warning(`${pushForm.pushKind === 'public' ? '公信' : '私信'}推送请选择模板`)
    return
  }

  if (pushForm.pushKind === 'public' || isPrivateFreeTemplate(selectedTemplate.value)) {
    if (!pushTitle || !pushBody) {
      ElMessage.warning('请填写标题和内容')
      return
    }
  }

  if (isPrivateComposeTemplate(selectedTemplate.value)) {
    const missingTitleVars = titleTemplateVars.value.filter(
      (name) => !pushForm.titleVariables[name]?.trim(),
    )
    const missingBodyVars = bodyTemplateVars.value.filter(
      (name) => !pushForm.bodyVariables[name]?.trim(),
    )
    if (missingTitleVars.length) {
      ElMessage.warning(`请填写标题变量：${missingTitleVars.join('、')}`)
      return
    }
    if (missingBodyVars.length) {
      ElMessage.warning(`请填写内容变量：${missingBodyVars.join('、')}`)
      return
    }
  }
  if (pushForm.click_type === 'open_page') {
    const activity = pushForm.activity.trim()
    if (!activity) {
      ElMessage.warning('请填写目标 Activity 全类名')
      return
    }
    if (!isFullyQualifiedActivity(activity)) {
      ElMessage.warning('Activity 须为全类名，如 com.example.app.OrderDetailActivity')
      return
    }
  }
  if (pushForm.click_type === 'open_web' && !pushForm.url.trim()) {
    ElMessage.warning('请填写网页 URL')
    return
  }
  const notifyIdRaw = pushForm.notify_id.trim()
  let notifyId: number | undefined
  if (notifyIdRaw) {
    const parsed = Number(notifyIdRaw)
    if (!Number.isInteger(parsed) || parsed < 0 || parsed > 2147483647) {
      ElMessage.warning('通知 ID 须为 0~2147483647 的整数')
      return
    }
    notifyId = parsed
  }
  let clickParams: Record<string, unknown> | undefined
  if (pushForm.click_type === 'open_page') {
    const built = buildClickParams()
    if (built.error) {
      ElMessage.warning(built.error)
      return
    }
    clickParams = built.params
  }
  sending.value = true
  try {
    const clickAction: ClickAction = {
      type: pushForm.click_type,
      activity: pushForm.click_type === 'open_page' ? pushForm.activity.trim() : undefined,
      url: pushForm.click_type === 'open_web' ? pushForm.url.trim() : undefined,
      params: clickParams,
    }
    const requestBody: Parameters<typeof sendPush>[1] = {
      payload: {},
      targets: { device_ids: pushForm.device_ids },
      click_action: clickAction,
    }
    if (hasTemplate) {
      requestBody.template_id = pushForm.template_id
    }
    if (pushForm.pushKind === 'public' || isPrivateFreeTemplate(selectedTemplate.value)) {
      requestBody.title = pushTitle
      requestBody.body = pushBody
    }
    if (isPrivateComposeTemplate(selectedTemplate.value)) {
      requestBody.title_variables = { ...pushForm.titleVariables }
      requestBody.body_variables = { ...pushForm.bodyVariables }
    }
    // 未覆盖时也传具体截止时间，便于与界面展示一致
    requestBody.cache_until = formatCacheUntil(
      pushForm.overrideCacheUntil && pushForm.cacheUntil
        ? pushForm.cacheUntil
        : effectiveCacheUntil.value,
    )
    if (notifyId !== undefined) {
      requestBody.notify_id = notifyId
    }
    pushResult.value = await sendPush(appId.value, requestBody)
    await loadJobs()
    ElMessage.success('推送已发送')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '发送失败')
  } finally {
    sending.value = false
  }
}

watch(
  () => route.params.id,
  (id) => {
    if (typeof id === 'string') loadAll()
  },
)

onMounted(loadAll)
</script>

<style scoped>
.workspace-panel {
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.workspace-section {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 24px;
}

.section-header {
  margin-bottom: 20px;
}

.section-header__title {
  margin: 0;
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--ph-text);
}

.section-header__desc {
  margin: 6px 0 0;
  font-size: 14px;
  color: var(--ph-text-muted);
  line-height: 1.5;
}

.config-form {
  max-width: 720px;
  padding-bottom: 8px;
}

.config-form :deep(.el-form-item) {
  margin-bottom: 20px;
}

.config-form :deep(.el-form-item:last-child) {
  margin-bottom: 0;
}

.config-block {
  margin-bottom: 28px;
  padding-bottom: 28px;
  border-bottom: 1px solid var(--ph-border);
}

.config-block:last-of-type {
  border-bottom: none;
  padding-bottom: 0;
}

.config-block__title {
  margin: 0 0 16px;
  font-size: 15px;
  font-weight: 700;
  color: var(--ph-text);
}

.config-block__head--vendor {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.config-block__head--vendor .config-block__title {
  margin: 0;
}

.credential-validations {
  display: grid;
  gap: 8px;
  margin-bottom: 12px;
}

.vendor-collapse-title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.config-block__desc {
  margin: -8px 0 14px;
  font-size: 13px;
  color: var(--ph-text-muted);
  line-height: 1.5;
}

.config-block__optional {
  font-size: 12px;
  font-weight: 600;
  color: var(--ph-text-muted);
}

.config-block__hint {
  margin: -4px 0 12px;
  font-size: 13px;
  color: var(--ph-text-muted);
  line-height: 1.5;
}

.config-block--online {
  margin-top: 0;
  margin-bottom: 24px;
}

.config-form__actions {
  margin-top: 0;
  padding-top: 24px;
  border-top: 1px solid var(--ph-border);
}

.vendor-collapse {
  border: 1px solid var(--ph-border);
  border-radius: var(--ph-radius-md);
  overflow: hidden;
  background: #fff;
}

.vendor-collapse :deep(.el-collapse-item) {
  border-bottom: 1px solid var(--ph-border);
}

.vendor-collapse :deep(.el-collapse-item:last-child) {
  border-bottom: none;
}

.vendor-collapse :deep(.el-collapse-item__header) {
  min-height: 48px;
  padding: 0 16px;
  line-height: 1.4;
  font-weight: 700;
  color: var(--ph-text);
  background: linear-gradient(180deg, #fafbff 0%, #fff 100%);
  border-bottom: none;
}

.vendor-collapse :deep(.el-collapse-item.is-active > .el-collapse-item__header) {
  border-bottom: 1px solid var(--ph-border);
}

.vendor-collapse :deep(.el-collapse-item__wrap) {
  border-bottom: none;
  background: #fff;
}

.vendor-collapse :deep(.el-collapse-item__content) {
  padding: 16px 16px 4px;
}

.vendor-collapse :deep(.el-form-item) {
  margin-bottom: 16px;
}

.vendor-collapse :deep(.el-form-item:last-child) {
  margin-bottom: 12px;
}

.stats-cards {
  margin-top: 4px;
}

.stats-group-title {
  margin: 20px 0 8px;
  font-size: 14px;
  font-weight: 700;
  color: var(--ph-text);
}

.stats-group-title:first-of-type {
  margin-top: 8px;
}

.stats-detail-row {
  margin-top: 16px;
}

.inner-card {
  border-radius: var(--ph-radius-md);
  border: 1px solid var(--ph-border);
}

.inner-card__title {
  font-weight: 700;
  color: var(--ph-text);
}

.push-result-alert {
  max-width: 800px;
  margin-top: 12px;
}

h4 {
  margin: 12px 0 8px;
  font-size: 14px;
  color: #303133;
}

.push-form {
  max-width: 800px;
}

.push-form :deep(.el-form-item) {
  margin-bottom: 24px;
}

.push-form :deep(.el-form-item__content) {
  flex-direction: column;
  align-items: stretch;
  line-height: normal;
}

.push-form :deep(.el-form-item__content > .el-select),
.push-form :deep(.el-form-item__content > .el-input),
.push-form :deep(.el-form-item__content > .el-textarea) {
  width: 100%;
}

.push-form .ph-field-hint {
  width: 100%;
  margin-top: 8px;
}

.click-params {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.click-params__row {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) 88px minmax(0, 1.4fr) auto;
  gap: 8px;
  align-items: center;
}

.click-params__type {
  width: 88px;
}

.click-params__value--bool {
  justify-self: start;
}

.click-params__remove {
  padding-inline: 4px;
}

.click-params__add {
  align-self: flex-start;
}

@media (max-width: 720px) {
  .click-params__row {
    grid-template-columns: minmax(0, 1fr) 88px auto;
  }

  .click-params__value,
  .click-params__value--bool {
    grid-column: 1 / -1;
  }

  .click-params__remove {
    grid-column: 3;
    grid-row: 1;
  }
}

.ph-inline-unit {
  margin-left: 8px;
  color: var(--ph-text-muted);
  font-size: 13px;
}

.push-kind-picker {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  width: 100%;
}

.push-kind-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  width: 100%;
  padding: 14px 16px;
  border: 1px solid rgba(15, 23, 42, 0.08);
  border-radius: var(--ph-radius-sm);
  background: #fff;
  text-align: left;
  cursor: pointer;
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.03);
  transition:
    border-color 0.16s ease,
    background 0.16s ease,
    box-shadow 0.16s ease,
    color 0.16s ease;
}

.push-kind-card:hover {
  border-color: rgba(99, 102, 241, 0.22);
  background: #fafbff;
}

.push-kind-card.is-active {
  border-color: rgba(99, 102, 241, 0.28);
  background: linear-gradient(180deg, #f8faff 0%, #eef2ff 100%);
  box-shadow:
    inset 3px 0 0 var(--ph-primary),
    0 1px 2px rgba(99, 102, 241, 0.06);
}

.push-kind-card__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--ph-text);
}

.push-kind-card.is-active .push-kind-card__title {
  color: var(--ph-primary-dark);
}

.push-kind-card__desc {
  font-size: 12px;
  line-height: 1.55;
  color: var(--ph-text-muted);
}

.push-kind-card.is-active .push-kind-card__desc {
  color: #64748b;
}

.template-compose-item :deep(.el-form-item__content) {
  line-height: normal;
}

.template-compose {
  width: 100%;
}

.template-compose__hint {
  margin: 10px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--ph-text-muted);
}

.msg-preview {
  padding: 16px 18px;
  border-radius: var(--ph-radius-md);
  border: 1px solid var(--ph-border);
  background: linear-gradient(180deg, #fafbff 0%, #fff 100%);
  box-shadow: var(--ph-shadow-sm);
}

.msg-preview__block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.msg-preview__tag {
  align-self: flex-start;
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.msg-preview__tag--title {
  background: var(--ph-primary-light);
  color: var(--ph-primary-dark);
}

.msg-preview__tag--body {
  background: var(--ph-accent-soft);
  color: #be185d;
}

.msg-preview__divider {
  height: 1px;
  margin: 14px 0;
  background: var(--ph-border);
}

.msg-preview__line {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 6px;
  word-break: break-word;
}

.msg-preview__line--title {
  font-size: 16px;
  font-weight: 700;
  line-height: 1.5;
  color: var(--ph-text);
}

.msg-preview__line--body {
  font-size: 14px;
  line-height: 1.65;
  color: var(--ph-text-muted);
  white-space: pre-wrap;
}

.msg-preview__text {
  display: inline-flex;
  align-items: center;
  white-space: pre-wrap;
  line-height: inherit;
}

.msg-preview__blank {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  margin: 0 1px;
  padding: 0 8px;
  min-height: 1.65em;
  border: 1.5px dashed rgba(99, 102, 241, 0.42);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.9);
  box-sizing: border-box;
  transition:
    border-color 0.15s ease,
    background 0.15s ease,
    box-shadow 0.15s ease;
}

.msg-preview__line--title .msg-preview__blank {
  min-height: 1.5em;
}

.msg-preview__line--body .msg-preview__blank {
  min-height: 1.65em;
}

.msg-preview__blank:focus-within {
  border-color: var(--ph-primary);
  border-style: dashed;
  background: #fff;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.msg-preview__input {
  field-sizing: content;
  min-width: 64px;
  max-width: min(260px, 100%);
  margin: 0;
  padding: 0;
  border: none;
  border-radius: 0;
  background: transparent;
  font: inherit;
  font-weight: 600;
  line-height: inherit;
  color: var(--ph-primary-dark);
  outline: none;
}

.msg-preview__line--title .msg-preview__input {
  font-size: 16px;
}

.msg-preview__line--body .msg-preview__input {
  font-size: 14px;
  font-weight: 500;
  color: var(--ph-text);
}

.msg-preview__input--body {
  min-width: 88px;
  max-width: 100%;
  resize: none;
  overflow: hidden;
  line-height: inherit;
}

.msg-preview__input::placeholder {
  color: rgba(99, 102, 241, 0.5);
  font-weight: 500;
}

.msg-preview__input:focus {
  outline: none;
}

.form-label-with-help {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.field-help-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--ph-text-muted);
  cursor: pointer;
  line-height: 1;
  transition: color 0.15s ease;
}

.field-help-btn:hover,
.field-help-btn:focus-visible {
  color: var(--ph-primary);
  outline: none;
}

.field-hint {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--ph-text-muted);
}

.section-header--with-action {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.vendor-switcher {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 4px;
  margin-bottom: 16px;
  border-radius: 12px;
  background: #f1f5f9;
  border: 1px solid rgba(15, 23, 42, 0.06);
}

.vendor-switcher__item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 34px;
  padding: 0 12px;
  border: 1px solid transparent;
  border-radius: 9px;
  background: transparent;
  color: var(--ph-text-muted);
  font-size: 13px;
  font-weight: 500;
  letter-spacing: 0.01em;
  cursor: pointer;
  transition:
    background 0.16s ease,
    color 0.16s ease,
    border-color 0.16s ease,
    box-shadow 0.16s ease;
}

.vendor-switcher__item:hover {
  color: var(--ph-text);
  background: rgba(255, 255, 255, 0.65);
}

.vendor-switcher__item.is-active {
  color: var(--ph-primary-dark);
  font-weight: 600;
  background: #fff;
  border-color: rgba(99, 102, 241, 0.14);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
}

.vendor-switcher__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  background: rgba(100, 116, 139, 0.1);
  color: var(--ph-text-muted);
}

.vendor-switcher__item.is-active .vendor-switcher__count {
  background: var(--ph-primary-light);
  color: var(--ph-primary-dark);
}

.vendor-panel {
  padding: 18px;
  border: 1px solid var(--ph-border);
  border-radius: var(--ph-radius-md, 14px);
  background: var(--ph-surface, #fff);
  box-shadow: var(--ph-shadow-sm);
}

.vendor-channel-body {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.ph-toolbar--inner {
  margin: 0;
  padding: 0;
}

.vendor-channel-block__head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.vendor-channel-block__head h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
}

.vivo-category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
}

.vivo-category-card {
  border: 1px solid var(--ph-border);
  border-radius: 12px;
  padding: 12px;
  background: var(--ph-bg, #fafafa);
}

.vivo-category-card--public {
  border-color: color-mix(in srgb, var(--el-color-success) 35%, var(--ph-border));
}

.vivo-category-card__name {
  font-size: 14px;
  font-weight: 700;
}

.vivo-category-card__code {
  margin-top: 4px;
  font-size: 12px;
  color: var(--ph-text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.vivo-category-card__desc {
  margin-top: 8px;
  font-size: 12px;
  line-height: 1.45;
  color: var(--ph-text-muted);
}

.vivo-private-checks {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
  width: 100%;
}

.vivo-private-check {
  margin: 0 !important;
  height: auto !important;
  padding: 12px 14px !important;
  align-items: flex-start;
  white-space: normal;
}

.vivo-private-check :deep(.el-checkbox__label) {
  display: flex;
  flex-direction: column;
  gap: 4px;
  white-space: normal;
  line-height: 1.4;
}

.vivo-private-check__label {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.vivo-private-check__label span {
  font-size: 12px;
  color: var(--ph-text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.vivo-private-check__desc {
  font-size: 12px;
  color: var(--ph-text-muted);
  font-weight: 400;
}

.job-message-trace {
  border: 1px solid var(--ph-border);
  border-radius: 12px;
  padding: 12px;
  margin-bottom: 12px;
  background: var(--ph-bg, #fafafa);
}

.job-message-trace__head {
  margin-bottom: 12px;
}

.job-message-trace__timeline {
  margin: 12px 0;
}

.job-message-trace__outbox {
  margin-top: 12px;
}

.job-message-trace__meta {
  margin-top: 6px;
  font-size: 12px;
  color: var(--ph-text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  word-break: break-all;
}

.job-message-trace__meta--error {
  color: var(--el-color-danger);
  font-family: inherit;
}

.field-help-btn .el-icon {
  font-size: 14px;
}

.vendor-guide-collapse {
  border: none;
}

.vendor-guide-collapse :deep(.el-collapse-item__header) {
  height: auto;
  min-height: 44px;
  line-height: 1.4;
  padding: 8px 0;
  border-bottom-color: rgba(99, 102, 241, 0.08);
}

.vendor-guide-collapse :deep(.el-collapse-item__wrap) {
  border-bottom-color: rgba(99, 102, 241, 0.08);
}

.vendor-guide-collapse :deep(.el-collapse-item__content) {
  padding: 8px 0 16px;
}

.vendor-guide-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
}

.vendor-guide-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.vendor-guide-steps {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.vendor-guide-steps__text {
  margin: 0 0 8px;
  color: var(--ph-text-muted);
  line-height: 1.65;
  font-size: 13px;
}

.vendor-guide-steps__text code {
  padding: 1px 6px;
  border-radius: 6px;
  background: var(--ph-primary-light);
  color: var(--ph-primary-dark);
  font-size: 12px;
}

.vendor-guide-tip {
  margin: 0;
  padding: 10px 12px;
  border-radius: 10px;
  background: #fff7ed;
  color: #9a3412;
  font-size: 13px;
  line-height: 1.5;
}
</style>
