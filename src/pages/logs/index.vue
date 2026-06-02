<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Delete, Search } from '@element-plus/icons-vue'
import { clearAppLogs, fetchAppLogs } from '@/api/settings'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { AppLogEntry } from '@/types'

const loading = ref(false)
const logs = ref<AppLogEntry[]>([])
const activeLevel = ref('all')
const keyword = ref('')
const activePreset = ref('all')
const activeIndex = ref(0)

const levelOptions = [
  { label: '全部', value: 'all' },
  { label: '错误', value: 'error' },
  { label: '警告', value: 'warn' },
  { label: '信息', value: 'info' },
  { label: '调试', value: 'debug' },
]

const presetOptions = [
  { label: '全部日志', value: 'all' },
  { label: '代理链路', value: 'proxy' },
  { label: '指纹启动', value: 'fingerprint' },
  { label: '浏览器启动', value: 'launch' },
  { label: '异常问题', value: 'issues' },
]

async function loadLogs() {
  loading.value = true
  try {
    logs.value = (await fetchAppLogs()).slice().reverse()
    if (activeIndex.value >= logs.value.length) {
      activeIndex.value = 0
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载日志失败')
  } finally {
    loading.value = false
  }
}

async function handleClear() {
  try {
    await clearAppLogs()
    logs.value = []
    activeIndex.value = 0
    ElMessage.success('日志已清空')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '清空日志失败')
  }
}

function normalizeLevel(level: string) {
  const value = level.trim().toLowerCase()
  if (value === 'warning') return 'warn'
  return value
}

function levelTagType(level: string) {
  switch (normalizeLevel(level)) {
    case 'error':
      return 'danger'
    case 'warn':
      return 'warning'
    case 'info':
      return 'info'
    case 'debug':
      return ''
    default:
      return 'info'
  }
}

function levelLabel(level: string) {
  switch (normalizeLevel(level)) {
    case 'error':
      return '错误'
    case 'warn':
      return '警告'
    case 'info':
      return '信息'
    case 'debug':
      return '调试'
    default:
      return level || '未知'
  }
}

function formatTime(value: string) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString('zh-CN', { hour12: false })
}

function matchPreset(item: AppLogEntry, preset: string) {
  const target = (item.target || '').toLowerCase()
  const level = normalizeLevel(item.level)

  switch (preset) {
    case 'proxy':
      return target.startsWith('proxy.') || target === 'browser.launch'
    case 'fingerprint':
      return target === 'browser.fingerprint'
    case 'launch':
      return target.startsWith('browser.') || target === 'launch-api'
    case 'issues':
      return level === 'error' || level === 'warn'
    default:
      return true
  }
}

const searchPlaceholder = computed(() => {
  switch (activePreset.value) {
    case 'proxy':
      return '搜索代理模式、bridge 端口、插件 host、source'
    case 'fingerprint':
      return '搜索品牌、平台、语言、时区、分辨率'
    case 'launch':
      return '搜索实例 ID、实例名、启动链路'
    case 'issues':
      return '搜索错误信息、警告详情、模块名'
    default:
      return '搜索模块、内容、详情、时间'
  }
})

const filteredLogs = computed(() => {
  const level = activeLevel.value
  const preset = activePreset.value
  const search = keyword.value.trim().toLowerCase()

  return logs.value.filter((item) => {
    if (!matchPreset(item, preset)) return false

    const matchLevel = level === 'all' || normalizeLevel(item.level) === level
    if (!matchLevel) return false
    if (!search) return true

    return [
      item.message,
      item.target,
      item.details,
      item.createdAt,
    ]
      .join('\n')
      .toLowerCase()
      .includes(search)
  })
})

const activeLog = computed(() => filteredLogs.value[activeIndex.value] ?? null)

const stats = computed(() => ({
  total: logs.value.length,
  errors: logs.value.filter((item) => normalizeLevel(item.level) === 'error').length,
  warns: logs.value.filter((item) => normalizeLevel(item.level) === 'warn').length,
  filtered: filteredLogs.value.length,
}))

function selectLog(index: number) {
  activeIndex.value = index
}

function previewMessage(item: AppLogEntry) {
  const source = item.details?.trim() || item.message || ''
  return source.length > 120 ? `${source.slice(0, 120)}...` : source
}

watch(filteredLogs, (items) => {
  if (items.length === 0) {
    activeIndex.value = 0
    return
  }
  if (activeIndex.value >= items.length) {
    activeIndex.value = 0
  }
})

onMounted(loadLogs)
usePageRefresh(loadLogs)
</script>

<template>
  <section class="logs-page">
    <div class="page-header">
      <div>
        <h1 class="page-title">日志查看</h1>
        <p class="page-subtitle">按模块、级别和关键字快速定位运行问题，右侧查看完整日志详情。</p>
      </div>
      <div class="page-actions">
        <el-button :icon="Delete" type="danger" plain @click="handleClear">清空日志</el-button>
      </div>
    </div>

    <div class="page-grid-4">
      <div class="metric-card">
        <div class="metric-card__label">总日志</div>
        <div class="metric-card__value">{{ stats.total }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">错误</div>
        <div class="metric-card__value">{{ stats.errors }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">警告</div>
        <div class="metric-card__value">{{ stats.warns }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">筛选结果</div>
        <div class="metric-card__value">{{ stats.filtered }}</div>
      </div>
    </div>

    <div class="content-panel logs-toolbar">
      <div class="logs-toolbar__groups">
        <el-segmented v-model="activePreset" :options="presetOptions" />
        <el-segmented v-model="activeLevel" :options="levelOptions" />
      </div>
      <el-input
        v-model="keyword"
        class="logs-toolbar__search"
        :prefix-icon="Search"
        clearable
        :placeholder="searchPlaceholder"
      />
    </div>

    <div class="logs-layout">
      <div class="content-panel logs-list-panel">
        <div class="table-panel__meta">
          <span class="muted-text">{{ filteredLogs.length }} 条匹配日志</span>
        </div>
        <div v-loading="loading" class="logs-list">
          <button
            v-for="(item, index) in filteredLogs"
            :key="`${item.createdAt}-${index}`"
            type="button"
            class="log-row"
            :class="{ 'log-row--active': index === activeIndex }"
            @click="selectLog(index)"
          >
            <div class="log-row__head">
              <el-tag :type="levelTagType(item.level)" effect="plain" size="small">
                {{ levelLabel(item.level) }}
              </el-tag>
              <span class="log-row__target">{{ item.target || 'app' }}</span>
              <time class="log-row__time">{{ formatTime(item.createdAt) }}</time>
            </div>
            <div class="log-row__message">{{ item.message || '-' }}</div>
            <div class="log-row__preview">{{ previewMessage(item) || '无附加详情' }}</div>
          </button>
          <el-empty v-if="!loading && filteredLogs.length === 0" description="暂无匹配日志" />
        </div>
      </div>

      <div class="content-panel logs-detail-panel">
        <template v-if="activeLog">
          <div class="logs-detail__head">
            <div>
              <div class="logs-detail__eyebrow">{{ activeLog.target || 'app' }}</div>
              <h2 class="logs-detail__title">{{ activeLog.message || '日志详情' }}</h2>
            </div>
            <el-tag :type="levelTagType(activeLog.level)" effect="plain">
              {{ levelLabel(activeLog.level) }}
            </el-tag>
          </div>

          <div class="logs-detail__grid">
            <div class="soft-card p-4">
              <div class="soft-card__label">时间</div>
              <div class="soft-card__text">{{ formatTime(activeLog.createdAt) }}</div>
            </div>
            <div class="soft-card p-4">
              <div class="soft-card__label">模块</div>
              <div class="soft-card__text">{{ activeLog.target || 'app' }}</div>
            </div>
          </div>

          <div class="logs-detail__section">
            <div class="logs-detail__label">摘要</div>
            <pre class="logs-detail__code">{{ activeLog.message || '-' }}</pre>
          </div>

          <div class="logs-detail__section">
            <div class="logs-detail__label">详细内容</div>
            <pre class="logs-detail__code">{{ activeLog.details || '无附加详情' }}</pre>
          </div>

          <div class="logs-detail__section">
            <div class="logs-detail__label">原始记录</div>
            <pre class="logs-detail__code">{{ JSON.stringify(activeLog, null, 2) }}</pre>
          </div>
        </template>

        <el-empty v-else description="选择左侧日志查看详情" />
      </div>
    </div>
  </section>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.logs-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.logs-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 18px;
}

.logs-toolbar__groups {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.logs-toolbar__search {
  max-width: 320px;
  margin-left: auto;
}

.logs-layout {
  display: grid;
  grid-template-columns: minmax(320px, 0.95fr) minmax(380px, 1.25fr);
  gap: 18px;
  min-height: 640px;
}

.logs-list-panel,
.logs-detail-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.logs-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px;
  min-height: 0;
  overflow-y: auto;
}

.log-row {
  width: 100%;
  padding: 14px 16px;
  @include glass-panel(rgba(255, 255, 255, 0.68), none);
  text-align: left;
  cursor: pointer;
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
}

.log-row:hover {
  border-color: rgba(15, 159, 122, 0.28);
  transform: translateY(-1px);
}

.log-row--active {
  border-color: rgba(15, 159, 122, 0.48);
  background: rgba(238, 251, 245, 0.82);
  box-shadow: var(--shadow-subtle);
}

.log-row__head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.log-row__target {
  font-size: 12px;
  font-weight: 700;
  color: var(--primary-strong);
}

.log-row__time {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-muted);
}

.log-row__message {
  margin-top: 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  line-height: 1.5;
}

.log-row__preview {
  margin-top: 8px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-soft);
  white-space: pre-wrap;
  word-break: break-word;
}

.logs-detail__head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  padding: 18px 18px 0;
}

.logs-detail__eyebrow {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
  color: var(--primary-strong);
}

.logs-detail__title {
  margin: 8px 0 0;
  font-size: 20px;
  line-height: 1.4;
  color: var(--text);
}

.logs-detail__grid {
  margin-top: 20px;
  padding: 0 18px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.logs-detail__section {
  margin-top: 18px;
  padding: 0 18px;
}

.logs-detail__label {
  margin-bottom: 10px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}

.logs-detail__code {
  margin: 0;
  padding: 14px;
  @include glass-panel(rgba(247, 252, 250, 0.86), none);
  color: var(--text);
  font-size: 12px;
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-x: auto;
}

@include respond-down($breakpoint-logs-stack) {
  .logs-layout {
    grid-template-columns: 1fr;
  }
}

@include respond-down(768px) {
  .logs-toolbar {
    flex-direction: column;
    align-items: stretch;
  }

  .logs-toolbar__groups {
    flex-direction: column;
    align-items: stretch;
  }

  .logs-toolbar__search {
    max-width: none;
    margin-left: 0;
  }

  .logs-detail__grid {
    grid-template-columns: 1fr;
  }
}
</style>
