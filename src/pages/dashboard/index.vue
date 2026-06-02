<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  CircleCheck,
  Cpu,
  Link,
  Monitor,
  Plus,
  Setting,
  VideoPlay,
  Warning,
} from '@element-plus/icons-vue'
import { fetchDashboardWorkbench } from '@/api/dashboard'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { DashboardWorkbench } from '@/types/dashboard'

type HealthTone = 'good' | 'warn' | 'quiet'

interface HealthCard {
  title: string
  value: string | number
  caption: string
  detail: string
  icon: unknown
  tone: HealthTone
  actionLabel: string
  actionPath: string
}

interface WorkItem {
  title: string
  description: string
  path: string
  action: string
  tone: HealthTone
}

const router = useRouter()
const loading = ref(false)
const workbench = ref<DashboardWorkbench | null>(null)

const stats = computed(() => workbench.value?.stats)
const system = computed(() => workbench.value?.system)
const defaultCore = computed(() => workbench.value?.defaultCore ?? null)
const automation = computed(() => workbench.value?.automation)
const runningProfiles = computed(() => workbench.value?.runningProfiles ?? [])

const overallState = computed(() => {
  const blockers = [
    (stats.value?.coreCount ?? 0) === 0,
    (stats.value?.proxyCount ?? 0) === 0,
    (stats.value?.invalidCoreCount ?? 0) > 0,
    Boolean(system.value && !system.value.mihomoDownloaded && system.value.proxyMode === 'mihomo'),
  ].filter(Boolean).length

  if (blockers > 0) {
    return {
      label: `${blockers} 项需要处理`,
      type: 'warning' as const,
      copy: '先处理内核、代理或运行时配置，启动实例会更稳定。',
    }
  }

  if ((stats.value?.runningInstances ?? 0) > 0) {
    return {
      label: '工作台运行中',
      type: 'success' as const,
      copy: `${stats.value?.runningInstances ?? 0} 个实例正在运行，代理和内核状态可继续观察。`,
    }
  }

  return {
    label: '准备就绪',
    type: 'success' as const,
    copy: '基础资源已经配置，可以创建或启动浏览器实例。',
  }
})

const healthCards = computed<HealthCard[]>(() => [
  {
    title: '浏览器实例',
    value: stats.value?.totalInstances ?? 0,
    caption: `${stats.value?.runningInstances ?? 0} 个运行中`,
    detail: (stats.value?.unboundInstances ?? 0) > 0 ? `${stats.value?.unboundInstances ?? 0} 个实例未绑定代理` : '实例网络出口配置完整',
    icon: Monitor,
    tone: (stats.value?.unboundInstances ?? 0) > 0 ? 'warn' : 'good',
    actionLabel: '管理实例',
    actionPath: '/browser/list',
  },
  {
    title: '代理池',
    value: stats.value?.proxyCount ?? 0,
    caption: `${stats.value?.healthyProxyCount ?? 0} 个最近检测可用`,
    detail: (stats.value?.untestedProxyCount ?? 0) > 0 ? `${stats.value?.untestedProxyCount ?? 0} 个节点尚未检测` : '代理检测状态清楚',
    icon: Link,
    tone: (stats.value?.proxyCount ?? 0) === 0 || (stats.value?.untestedProxyCount ?? 0) > 0 ? 'warn' : 'good',
    actionLabel: '查看代理',
    actionPath: '/browser/proxy-pool',
  },
  {
    title: '浏览器内核',
    value: stats.value?.coreCount ?? 0,
    caption: defaultCore.value?.coreName || '未设置默认内核',
    detail: (stats.value?.invalidCoreCount ?? 0) > 0 ? `${stats.value?.invalidCoreCount ?? 0} 个内核路径异常` : '默认内核可用于新实例',
    icon: Cpu,
    tone: (stats.value?.coreCount ?? 0) === 0 || (stats.value?.invalidCoreCount ?? 0) > 0 ? 'warn' : 'good',
    actionLabel: '管理内核',
    actionPath: '/browser/cores',
  },
  {
    title: '自动化环境',
    value: automation.value?.ready ? '就绪' : '未就绪',
    caption: `${stats.value?.scriptCount ?? 0} 个脚本 / ${stats.value?.runCount ?? 0} 条记录`,
    detail: automation.value?.lastError || (automation.value?.ready ? '运行时已经准备完成' : '运行时还未完成安装'),
    icon: VideoPlay,
    tone: automation.value?.ready ? 'good' : 'quiet',
    actionLabel: '查看自动化',
    actionPath: '/browser/automation',
  },
])

const workItems = computed<WorkItem[]>(() => {
  const items: WorkItem[] = []
  const current = stats.value
  const currentSystem = system.value

  if ((current?.coreCount ?? 0) === 0) {
    items.push({
      title: '先添加浏览器内核',
      description: '没有内核时无法稳定创建和启动实例。',
      path: '/browser/cores',
      action: '添加内核',
      tone: 'warn',
    })
  }

  if ((current?.proxyCount ?? 0) === 0) {
    items.push({
      title: '配置代理池',
      description: '代理池为空，实例只能使用直连或手动参数。',
      path: '/browser/proxy-pool',
      action: '添加代理',
      tone: 'warn',
    })
  } else if ((current?.untestedProxyCount ?? 0) > 0) {
    items.push({
      title: '检测代理可用性',
      description: `${current?.untestedProxyCount ?? 0} 个节点还没有检测记录，建议先批量检测。`,
      path: '/browser/proxy-pool',
      action: '去检测',
      tone: 'warn',
    })
  }

  if ((current?.unboundInstances ?? 0) > 0) {
    items.push({
      title: '检查实例网络出口',
      description: `${current?.unboundInstances ?? 0} 个实例未绑定代理，可能不符合隔离预期。`,
      path: '/browser/list',
      action: '查看实例',
      tone: 'quiet',
    })
  }

  if ((current?.invalidCoreCount ?? 0) > 0) {
    items.push({
      title: '修复异常内核路径',
      description: `${current?.invalidCoreCount ?? 0} 个内核路径不可用，启动前需要修复。`,
      path: '/browser/cores',
      action: '修复内核',
      tone: 'warn',
    })
  }

  if (currentSystem && currentSystem.proxyMode === 'mihomo' && !currentSystem.mihomoDownloaded) {
    items.push({
      title: '下载 Mihomo 运行时',
      description: '当前代理模式依赖 Mihomo，但运行时还没有下载。',
      path: '/settings',
      action: '去设置',
      tone: 'warn',
    })
  }

  if (items.length === 0) {
    items.push({
      title: '配置状态良好',
      description: '内核、代理和实例基础状态都已经就绪，可以继续日常操作。',
      path: '/browser/list',
      action: '进入实例列表',
      tone: 'good',
    })
  }

  return items.slice(0, 5)
})

function runtimeProxyKind(summary: string) {
  const value = (summary || '').trim()
  if (!value || value === '未运行') return '未运行'
  if (value.startsWith('Mihomo bridge')) return 'Mihomo'
  if (value.startsWith('插件 ')) return '插件代理'
  if (value.startsWith('直连')) return '直连'
  return '未知'
}

function formatTime(value?: string) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString('zh-CN', { hour12: false })
}

async function loadWorkbench() {
  loading.value = true
  try {
    workbench.value = await fetchDashboardWorkbench()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载今日工作台失败')
  } finally {
    loading.value = false
  }
}

onMounted(loadWorkbench)
usePageRefresh(loadWorkbench)
</script>

<template>
  <section class="workbench-page">
    <div class="workbench-hero content-panel">
      <div class="workbench-hero__copy">
        <div class="workbench-hero__eyebrow">今日工作台</div>
        <h1 class="workbench-hero__title">浏览器环境工作台</h1>
        <p class="workbench-hero__subtitle">{{ overallState.copy }}</p>
        <div class="workbench-hero__status">
          <el-tag :type="overallState.type" effect="plain">{{ overallState.label }}</el-tag>
          <span>{{ stats?.totalInstances ?? 0 }} 个实例</span>
          <span>{{ stats?.proxyCount ?? 0 }} 个代理</span>
          <span>{{ stats?.coreCount ?? 0 }} 个内核</span>
        </div>
      </div>

      <div class="workbench-hero__actions">
        <el-button type="primary" :icon="Plus" @click="router.push('/browser/list')">新建实例</el-button>
        <el-button :icon="Link" @click="router.push('/browser/proxy-pool')">添加代理</el-button>
        <el-button :icon="Cpu" @click="router.push('/browser/cores')">添加内核</el-button>
      </div>
    </div>

    <div class="workbench-health">
      <div
        v-for="card in healthCards"
        :key="card.title"
        class="health-card"
        :class="`health-card--${card.tone}`"
      >
        <div class="health-card__top">
          <span class="health-card__icon">
            <el-icon :size="18">
              <component :is="card.icon" />
            </el-icon>
          </span>
          <el-icon v-if="card.tone === 'good'" class="health-card__state"><CircleCheck /></el-icon>
          <el-icon v-else-if="card.tone === 'warn'" class="health-card__state"><Warning /></el-icon>
        </div>
        <div class="health-card__label">{{ card.title }}</div>
        <div class="health-card__value">{{ card.value }}</div>
        <div class="health-card__caption">{{ card.caption }}</div>
        <div class="health-card__detail">{{ card.detail }}</div>
        <el-button link type="primary" @click="router.push(card.actionPath)">{{ card.actionLabel }}</el-button>
      </div>
    </div>

    <div class="workbench-layout">
      <div class="content-panel workbench-panel">
        <div class="workbench-panel__head">
          <div>
            <h2 class="section-title">运行中实例</h2>
            <p class="section-copy">正在占用本机资源的浏览器实例会显示在这里，方便快速确认网络出口和调试端口。</p>
          </div>
          <el-button :icon="Monitor" @click="router.push('/browser/list')">实例列表</el-button>
        </div>

        <div v-if="runningProfiles.length" class="running-list">
          <div v-for="profile in runningProfiles" :key="profile.profileId" class="running-row">
            <div class="running-row__main">
              <div class="running-row__name">{{ profile.profileName }}</div>
              <div class="running-row__meta">
                <span>端口 {{ profile.debugPort || '-' }}</span>
                <span>{{ runtimeProxyKind(profile.runtimeProxySummary) }}</span>
                <span>更新 {{ formatTime(profile.updatedAt) }}</span>
              </div>
            </div>
            <el-tag type="success" effect="plain">运行中</el-tag>
          </div>
        </div>

        <div v-else class="empty-workbench">
          <el-empty description="当前没有运行中的实例">
            <el-button type="primary" @click="router.push('/browser/list')">启动或新建实例</el-button>
          </el-empty>
        </div>
      </div>

      <div class="page-stack">
        <div class="content-panel workbench-panel">
          <div class="workbench-panel__head">
            <div>
              <h2 class="section-title">待处理事项</h2>
              <p class="section-copy">根据当前资源状态自动生成，优先处理警告项。</p>
            </div>
          </div>

          <div class="work-list">
            <button
              v-for="item in workItems"
              :key="item.title"
              type="button"
              class="work-item"
              :class="`work-item--${item.tone}`"
              @click="router.push(item.path)"
            >
              <span class="work-item__dot" />
              <span class="work-item__body">
                <strong>{{ item.title }}</strong>
                <small>{{ item.description }}</small>
              </span>
              <span class="work-item__action">{{ item.action }}</span>
            </button>
          </div>
        </div>

        <div class="content-panel workbench-panel">
          <div class="workbench-panel__head">
            <div>
              <h2 class="section-title">系统信息</h2>
              <p class="section-copy">当前桌面端运行环境。</p>
            </div>
            <el-button :icon="Setting" circle @click="router.push('/settings')" />
          </div>

          <div class="system-list">
            <div>
              <span>应用版本</span>
              <strong>{{ system?.appVersion || 'unknown' }}</strong>
            </div>
            <div>
              <span>系统</span>
              <strong>{{ system?.os || 'unknown' }} / {{ system?.arch || 'unknown' }}</strong>
            </div>
            <div>
              <span>数据目录</span>
              <strong>{{ system?.appRoot || '-' }}</strong>
            </div>
            <div>
              <span>Mihomo</span>
              <strong>{{ system?.mihomoDownloaded ? '已下载' : '未下载' }}</strong>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.workbench-page {
  display: grid;
  gap: 18px;
}

.workbench-hero {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 24px;
  padding: 24px;
  overflow: hidden;
  background:
    radial-gradient(circle at 86% 22%, rgba(105, 207, 182, 0.16), transparent 34%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.9), rgba(244, 251, 249, 0.78));
}

.workbench-hero__eyebrow {
  color: var(--primary-strong);
  font-size: 12px;
  font-weight: 750;
}

.workbench-hero__title {
  margin: 8px 0 0;
  color: var(--text);
  font-size: 30px;
  font-weight: 800;
  line-height: 38px;
}

.workbench-hero__subtitle {
  max-width: 760px;
  margin: 10px 0 0;
  color: var(--text-soft);
  font-size: 14px;
  line-height: 1.7;
}

.workbench-hero__status {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin-top: 18px;
  color: var(--text-soft);
  font-size: 12px;
}

.workbench-hero__actions {
  display: flex;
  align-content: flex-start;
  align-items: flex-start;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
  max-width: 320px;
}

.workbench-health {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}

.health-card {
  min-width: 0;
  padding: 18px;
  @include glass-panel;
}

.health-card--warn {
  border-color: rgba(201, 133, 27, 0.32);
}

.health-card__top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.health-card__icon {
  display: grid;
  width: 38px;
  height: 38px;
  place-items: center;
  border-radius: 8px;
  background: var(--primary-soft);
  color: var(--primary-strong);
}

.health-card--warn .health-card__icon {
  background: rgba(255, 247, 224, 0.95);
  color: var(--amber);
}

.health-card__state {
  color: var(--primary);
}

.health-card--warn .health-card__state {
  color: var(--amber);
}

.health-card__label {
  margin-top: 16px;
  color: var(--text-soft);
  font-size: 12px;
  font-weight: 750;
}

.health-card__value {
  margin-top: 8px;
  color: var(--text);
  font-size: 28px;
  font-weight: 800;
  line-height: 34px;
}

.health-card__caption,
.health-card__detail {
  margin-top: 8px;
  color: var(--text-soft);
  font-size: 12px;
  line-height: 1.55;
}

.health-card__detail {
  min-height: 38px;
}

.workbench-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: 18px;
  align-items: start;
}

.workbench-panel {
  padding: 18px;
}

.workbench-panel__head {
  display: flex;
  justify-content: space-between;
  gap: 14px;
  align-items: flex-start;
  margin-bottom: 16px;
}

.running-list,
.work-list,
.system-list {
  display: grid;
  gap: 10px;
}

.running-row,
.work-item,
.system-list > div {
  border: 1px solid rgba(131, 151, 160, 0.16);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.68);
}

.running-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 14px;
}

.running-row__name {
  color: var(--text);
  font-size: 14px;
  font-weight: 750;
}

.running-row__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 5px;
  color: var(--text-soft);
  font-size: 12px;
}

.empty-workbench {
  display: grid;
  min-height: 300px;
  place-items: center;
}

.work-item {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 10px;
  width: 100%;
  padding: 12px;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.18s ease, transform 0.18s ease;
}

.work-item:hover {
  border-color: rgba(15, 159, 122, 0.3);
  transform: translateY(-1px);
}

.work-item__dot {
  width: 9px;
  height: 9px;
  margin-top: 5px;
  border-radius: 999px;
  background: var(--primary);
}

.work-item--warn .work-item__dot {
  background: var(--amber);
}

.work-item__body {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.work-item__body strong {
  color: var(--text);
  font-size: 13px;
}

.work-item__body small {
  color: var(--text-soft);
  font-size: 12px;
  line-height: 1.5;
}

.work-item__action {
  align-self: center;
  color: var(--primary-strong);
  font-size: 12px;
  font-weight: 750;
}

.system-list > div {
  display: grid;
  gap: 5px;
  padding: 12px;
}

.system-list span {
  color: var(--text-soft);
  font-size: 12px;
}

.system-list strong {
  color: var(--text);
  font-size: 13px;
  line-height: 1.5;
  overflow-wrap: anywhere;
}

@include wide-compact {
  .workbench-health {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@include two-column-stack {
  .workbench-layout,
  .workbench-hero {
    grid-template-columns: 1fr;
  }

  .workbench-hero__actions {
    justify-content: flex-start;
    max-width: none;
  }
}

@include mobile {
  .workbench-health {
    grid-template-columns: 1fr;
  }

  .workbench-hero {
    padding: 18px;
  }

  .workbench-hero__title {
    font-size: 24px;
    line-height: 32px;
  }

  .workbench-panel__head,
  .running-row {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
