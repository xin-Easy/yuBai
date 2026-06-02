<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { CircleCheck, CircleClose, Delete, Link, Plus, Refresh, Upload, View } from '@element-plus/icons-vue'
import {
  batchTestBrowserProxies,
  checkBrowserProxyIPHealth,
  deleteBrowserProxy,
  fetchBrowserProxies,
  fetchClashSubscription,
  importBrowserProxies,
  listBrowserProxiesBySource,
  refreshBrowserProxySubscription,
  saveBrowserProxy,
  testBrowserProxy,
} from '@/api/proxies'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { BrowserProxy } from '@/types'

const SYSTEM_DIRECT_PROXY_ID = 'proxy-direct'

const loading = ref(false)
const saving = ref(false)
const testingAll = ref(false)
const dialogOpen = ref(false)
const fetchingSubscription = ref(false)
const importing = ref(false)
const testingIds = ref<Set<string>>(new Set())
const proxies = ref<BrowserProxy[]>([])

const pageTab = ref<'manual' | 'subscription'>('manual')

const dialogTab = ref<'standard' | 'quick' | 'clash'>('standard')

const standardForm = reactive({
  proxyId: '',
  name: '',
  protocol: 'http',
  host: '',
  port: '8080',
  username: '',
  password: '',
  dnsServers: '',
  groupName: '',
})

const quickForm = reactive({
  content: '',
  url: '',
  mode: 'text' as 'text' | 'url',
  groupName: '',
  namePrefix: '',
  dnsServers: '',
})

const clashForm = reactive({
  url: '',
  groupName: '',
  namePrefix: '',
  dnsServers: '',
  fetchedContent: '',
  fetchedProxyCount: 0,
  fetchedSuggestedGroup: '',
  fetchedDns: '',
})

const quickPlaceholder = '每行输入一个代理，支持以下格式：\nsocks5://user:pass@192.168.1.1:1080\nhttp://192.168.1.1:8080\nhttps://user:pass@proxy.example.com:443'

const dialogTabOptions = [
  { label: '标准添加', value: 'standard' },
  { label: '快捷添加', value: 'quick' },
  { label: 'Clash 订阅', value: 'clash' },
]

const manualProxies = computed(() => proxies.value.filter((p) => !p.sourceId))
const subscriptionProxies = computed(() => proxies.value.filter((p) => p.sourceId))

interface SubscriptionGroup {
  sourceId: string
  sourceUrl: string
  groupName: string
  count: number
  healthyCount: number
  lastRefreshAt: string
}

const subscriptionGroups = computed<SubscriptionGroup[]>(() => {
  const map = new Map<string, BrowserProxy[]>()
  for (const p of subscriptionProxies.value) {
    const key = p.sourceId || ''
    if (!map.has(key)) map.set(key, [])
    map.get(key)!.push(p)
  }
  const groups: SubscriptionGroup[] = []
  for (const [, items] of map) {
    const first = items[0]
    groups.push({
      sourceId: first.sourceId || '',
      sourceUrl: first.sourceUrl || '',
      groupName: first.groupName || '',
      count: items.length,
      healthyCount: items.filter((p) => p.lastTestOk).length,
      lastRefreshAt: first.sourceLastRefreshAt || '',
    })
  }
  return groups
})

const healthyCount = computed(() => proxies.value.filter((item) => item.lastTestOk).length)
const groupCount = computed(() => new Set(proxies.value.map((item) => item.groupName).filter(Boolean)).size)

const detailOpen = ref(false)
const detailSourceId = ref('')
const detailProxies = ref<BrowserProxy[]>([])
const detailLoading = ref(false)
const detailRefreshing = ref(false)

function buildProxyConfig(): string {
  const { protocol, host, port, username, password } = standardForm
  if (!host.trim()) return ''
  let auth = ''
  if (username) auth = `${encodeURIComponent(username)}:${encodeURIComponent(password)}@`
  return `${protocol.toLowerCase()}://${auth}${host.trim()}:${port}`
}

async function loadProxies() {
  loading.value = true
  try {
    proxies.value = await fetchBrowserProxies()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载代理列表失败')
  } finally {
    loading.value = false
  }
}

function openCreateDialog(tab?: 'standard' | 'quick' | 'clash') {
  dialogTab.value = tab || 'standard'
  standardForm.proxyId = ''
  standardForm.name = ''
  standardForm.protocol = 'http'
  standardForm.host = ''
  standardForm.port = '8080'
  standardForm.username = ''
  standardForm.password = ''
  standardForm.dnsServers = ''
  standardForm.groupName = ''

  quickForm.mode = 'text'
  quickForm.content = ''
  quickForm.url = ''
  quickForm.groupName = ''
  quickForm.namePrefix = ''
  quickForm.dnsServers = ''

  clashForm.url = ''
  clashForm.groupName = ''
  clashForm.namePrefix = ''
  clashForm.dnsServers = ''
  clashForm.fetchedContent = ''
  clashForm.fetchedProxyCount = 0
  clashForm.fetchedSuggestedGroup = ''
  clashForm.fetchedDns = ''

  dialogOpen.value = true
}

async function handleSaveStandard() {
  if (!standardForm.name.trim()) {
    ElMessage.warning('请填写代理名称')
    return
  }
  if (!standardForm.host.trim()) {
    ElMessage.warning('请填写主机地址')
    return
  }

  saving.value = true
  try {
    const saved = await saveBrowserProxy({
      proxyId: standardForm.proxyId?.trim(),
      proxyName: standardForm.name.trim(),
      proxyConfig: buildProxyConfig(),
      dnsServers: standardForm.dnsServers?.trim(),
      groupName: standardForm.groupName?.trim(),
    })
    upsertProxy(saved)
    dialogOpen.value = false
    ElMessage.success('代理已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存代理失败')
  } finally {
    saving.value = false
  }
}

async function handleFetchSubscription() {
  if (!quickForm.url.trim()) {
    ElMessage.warning('请填写订阅 URL')
    return
  }

  fetchingSubscription.value = true
  try {
    const result = await fetchClashSubscription(quickForm.url.trim())
    quickForm.content = result.content
    quickForm.groupName = quickForm.groupName || result.suggestedGroup || ''
    quickForm.dnsServers = quickForm.dnsServers || result.dnsServers || ''
    ElMessage.success(`已拉取 ${result.proxyCount} 个节点`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '拉取订阅失败')
  } finally {
    fetchingSubscription.value = false
  }
}

async function handleImportQuick() {
  if (!quickForm.content.trim()) {
    ElMessage.warning('请粘贴 URI 或 Clash YAML 内容')
    return
  }

  importing.value = true
  try {
    const result = await importBrowserProxies({
      content: quickForm.content,
      groupName: quickForm.groupName.trim(),
      namePrefix: quickForm.namePrefix.trim(),
      dnsServers: quickForm.dnsServers.trim(),
    })
    result.items.forEach(upsertProxy)
    dialogOpen.value = false
    ElMessage.success(`导入 ${result.imported} 个，跳过 ${result.skipped} 个，失败 ${result.failed} 个`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '导入代理失败')
  } finally {
    importing.value = false
  }
}

async function handleClashFetch() {
  if (!clashForm.url.trim()) {
    ElMessage.warning('请填写订阅 URL')
    return
  }

  fetchingSubscription.value = true
  try {
    const result = await fetchClashSubscription(clashForm.url.trim())
    clashForm.fetchedContent = result.content
    clashForm.fetchedProxyCount = result.proxyCount
    clashForm.fetchedSuggestedGroup = result.suggestedGroup || ''
    clashForm.fetchedDns = result.dnsServers || ''
    if (!clashForm.groupName && result.suggestedGroup) {
      clashForm.groupName = result.suggestedGroup
    }
    if (!clashForm.dnsServers && result.dnsServers) {
      clashForm.dnsServers = result.dnsServers
    }
    ElMessage.success(`已解析 ${result.proxyCount} 个节点`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '拉取订阅失败')
  } finally {
    fetchingSubscription.value = false
  }
}

async function handleClashImport() {
  if (!clashForm.fetchedContent) {
    ElMessage.warning('请先拉取订阅内容')
    return
  }

  importing.value = true
  try {
    const sourceId = `sub-${Date.now()}`
    const result = await importBrowserProxies({
      content: clashForm.fetchedContent,
      groupName: clashForm.groupName.trim(),
      namePrefix: clashForm.namePrefix.trim(),
      dnsServers: clashForm.dnsServers.trim(),
      sourceId,
      sourceUrl: clashForm.url.trim(),
      sourceNamePrefix: clashForm.namePrefix.trim(),
    })
    result.items.forEach(upsertProxy)
    dialogOpen.value = false
    ElMessage.success(`导入 ${result.imported} 个，跳过 ${result.skipped} 个，失败 ${result.failed} 个`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '导入订阅失败')
  } finally {
    importing.value = false
  }
}

async function handleTestProxy(row: BrowserProxy, ipHealth = false) {
  if (isSystemDirectProxy(row)) {
    ElMessage.info('本机直连不需要检测')
    return
  }
  setTesting(row.proxyId, true)
  try {
    const result = ipHealth
      ? await checkBrowserProxyIPHealth(row.proxyId)
      : await testBrowserProxy(row.proxyId)
    await loadProxies()
    if (result.ok) {
      ElMessage.success(`${row.proxyName} 检测通过，${result.latencyMs} ms`)
    } else {
      ElMessage.error(result.error || `${row.proxyName} 检测失败`)
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '代理检测失败')
  } finally {
    setTesting(row.proxyId, false)
  }
}

async function handleBatchTest() {
  const source = pageTab.value === 'manual' ? manualProxies.value : proxies.value
  const targets = source.filter((item) => !isSystemDirectProxy(item))
  if (targets.length === 0) {
    ElMessage.info('暂无可检测代理')
    return
  }
  testingAll.value = true
  try {
    const result = await batchTestBrowserProxies(targets.map((item) => item.proxyId))
    await loadProxies()
    ElMessage.success(`检测完成：成功 ${result.ok} 个，失败 ${result.failed} 个`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '批量检测失败')
  } finally {
    testingAll.value = false
  }
}

async function handleDeleteProxy(row: BrowserProxy) {
  if (isSystemDirectProxy(row)) {
    ElMessage.info('系统内置直连代理不可删除')
    return
  }
  await ElMessageBox.confirm(`确认删除代理"${row.proxyName}"吗？`, '删除确认', {
    type: 'warning',
    confirmButtonText: '删除',
    cancelButtonText: '取消',
  })

  try {
    await deleteBrowserProxy(row.proxyId)
    proxies.value = proxies.value.filter((item) => item.proxyId !== row.proxyId)
    ElMessage.success('代理已删除')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '删除代理失败')
  }
}

async function openSubscriptionDetail(group: SubscriptionGroup) {
  detailSourceId.value = group.sourceId
  detailOpen.value = true
  detailLoading.value = true
  try {
    detailProxies.value = await listBrowserProxiesBySource(group.sourceId)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载订阅详情失败')
  } finally {
    detailLoading.value = false
  }
}

async function handleRefreshSubscription(group: SubscriptionGroup) {
  detailRefreshing.value = true
  try {
    const result = await refreshBrowserProxySubscription(group.sourceId)
    await loadProxies()
    detailProxies.value = await listBrowserProxiesBySource(group.sourceId)
    ElMessage.success(`更新完成：导入 ${result.imported} 个，跳过 ${result.skipped} 个，失败 ${result.failed} 个`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '更新订阅失败')
  } finally {
    detailRefreshing.value = false
  }
}

async function handleDeleteSubscription(group: SubscriptionGroup) {
  await ElMessageBox.confirm(
    `确认删除订阅「${group.sourceUrl}」及其 ${group.count} 个代理节点吗？`,
    '删除确认',
    { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' },
  )
  try {
    const ids = subscriptionProxies.value
      .filter((p) => p.sourceId === group.sourceId)
      .map((p) => p.proxyId)
    for (const id of ids) {
      await deleteBrowserProxy(id)
    }
    proxies.value = proxies.value.filter((p) => p.sourceId !== group.sourceId)
    if (detailOpen.value && detailSourceId.value === group.sourceId) {
      detailOpen.value = false
    }
    ElMessage.success('订阅已删除')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '删除订阅失败')
  }
}

function upsertProxy(proxy: BrowserProxy) {
  const existed = proxies.value.some((item) => item.proxyId === proxy.proxyId)
  proxies.value = existed
    ? proxies.value.map((item) => (item.proxyId === proxy.proxyId ? proxy : item))
    : [proxy, ...proxies.value]
}

function setTesting(proxyId: string, value: boolean) {
  const next = new Set(testingIds.value)
  if (value) {
    next.add(proxyId)
  } else {
    next.delete(proxyId)
  }
  testingIds.value = next
}

function proxyHealth(row: BrowserProxy) {
  if (isSystemDirectProxy(row)) {
    return { label: '直连', type: 'success' as const, icon: CircleCheck }
  }
  if (!row.lastTestedAt) {
    return { label: '未测', type: 'info' as const, icon: CircleClose }
  }
  if (row.lastTestOk) {
    return { label: '可用', type: 'success' as const, icon: CircleCheck }
  }
  return { label: '失败', type: 'danger' as const, icon: CircleClose }
}

function isSystemDirectProxy(row: BrowserProxy) {
  return row.proxyId === SYSTEM_DIRECT_PROXY_ID
}

function formatIPHealth(row: BrowserProxy) {
  if (!row.lastIPHealthJson) {
    return ''
  }
  try {
    const payload = JSON.parse(row.lastIPHealthJson)
    return payload.ip || payload.query || payload.address || payload.country || row.lastIPHealthJson
  } catch {
    return row.lastIPHealthJson
  }
}

function formatTime(iso?: string) {
  if (!iso) return '-'
  try {
    return new Date(iso).toLocaleString('zh-CN')
  } catch {
    return iso
  }
}

onMounted(loadProxies)
usePageRefresh(loadProxies)
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">代理池配置</h1>
        <p class="page-subtitle">导入、检测和维护实例要使用的代理节点，尽量把日常排查动作收进一个表里。</p>
      </div>
      <div class="page-actions">
        <el-button :icon="CircleCheck" :loading="testingAll" @click="handleBatchTest">批量检测</el-button>
        <el-button :icon="Upload" @click="openCreateDialog('quick')">快捷导入</el-button>
        <el-button type="primary" :icon="Plus" @click="openCreateDialog">添加代理</el-button>
      </div>
    </div>

    <div class="page-grid-3 proxy-summary">
      <div class="metric-card">
        <div class="metric-card__label">节点总数</div>
        <div class="metric-card__value">{{ proxies.length }}</div>
        <div class="metric-card__text">手动与订阅节点</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">检测通过</div>
        <div class="metric-card__value">{{ healthyCount }}</div>
        <div class="metric-card__text">最近一次可用</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">分组数量</div>
        <div class="metric-card__value">{{ groupCount }}</div>
        <div class="metric-card__text">按地区或用途归档</div>
      </div>
    </div>

    <div class="content-panel proxy-content">
      <el-tabs v-model="pageTab" class="proxy-tabs">
        <el-tab-pane label="标准代理" name="manual">
          <div class="proxy-tab__body">
            <div class="flex items-center justify-between mb-3">
              <span class="muted-text">共 {{ manualProxies.length }} 个标准代理</span>
            </div>

            <el-table v-loading="loading" :data="manualProxies" empty-text="暂无标准代理数据">
            <el-table-column label="代理名称" min-width="210" show-overflow-tooltip>
              <template #default="{ row }">
                <div class="proxy-name-cell">
                  <span>{{ row.proxyName }}</span>
                  <el-tag v-if="isSystemDirectProxy(row)" size="small" effect="plain" type="success">系统内置</el-tag>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="分组" width="140" show-overflow-tooltip>
              <template #default="{ row }">
                <span>{{ isSystemDirectProxy(row) ? '-' : row.groupName || '-' }}</span>
              </template>
            </el-table-column>
            <el-table-column label="配置" min-width="300" show-overflow-tooltip>
              <template #default="{ row }">
                <span>{{ isSystemDirectProxy(row) ? 'direct' : row.proxyConfig }}</span>
              </template>
            </el-table-column>
            <el-table-column label="绑定实例" min-width="150" show-overflow-tooltip>
              <template #default="{ row }">
                <div v-if="!isSystemDirectProxy(row) && row.boundProfileNames?.length" class="tag-cloud">
                  <el-tag v-for="name in row.boundProfileNames.slice(0, 3)" :key="name" effect="plain" size="small">{{ name }}</el-tag>
                  <span v-if="row.boundProfileNames.length > 3" class="muted-text">+{{ row.boundProfileNames.length - 3 }}</span>
                </div>
                <span v-else-if="isSystemDirectProxy(row)" class="muted-text">不适用</span>
                <span v-else class="muted-text">-</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="proxyHealth(row).type" effect="light">
                  <el-icon class="mr-1"><component :is="proxyHealth(row).icon" /></el-icon>
                  {{ proxyHealth(row).label }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="延迟" width="90">
              <template #default="{ row }">
                <span v-if="isSystemDirectProxy(row)" class="muted-text">不适用</span>
                <span v-else-if="row.lastLatencyMs >= 0">{{ row.lastLatencyMs }} ms</span>
                <span v-else class="muted-text">未测</span>
              </template>
            </el-table-column>
            <el-table-column label="出口信息" min-width="140" show-overflow-tooltip>
              <template #default="{ row }">
                <span>{{ isSystemDirectProxy(row) ? '不适用' : formatIPHealth(row) || '-' }}</span>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="240" fixed="right">
              <template #default="{ row }">
                <el-button
                  size="small"
                  plain
                  :loading="testingIds.has(row.proxyId)"
                  :disabled="isSystemDirectProxy(row)"
                  @click="handleTestProxy(row)"
                >
                  测试
                </el-button>
                <el-button
                  size="small"
                  plain
                  :icon="Link"
                  :loading="testingIds.has(row.proxyId)"
                  :disabled="isSystemDirectProxy(row)"
                  @click="handleTestProxy(row, true)"
                >
                  IP健康
                </el-button>
                <el-button
                  size="small"
                  type="danger"
                  plain
                  :disabled="isSystemDirectProxy(row)"
                  @click="handleDeleteProxy(row)"
                >
                  删除
                </el-button>
              </template>
            </el-table-column>
          </el-table>
          </div>
        </el-tab-pane>

        <el-tab-pane label="订阅链接" name="subscription">
          <div class="proxy-tab__body">
          <div v-if="subscriptionGroups.length === 0" class="soft-card p-8 text-center">
            <div class="muted-text mb-2">暂无订阅链接</div>
            <div class="muted-text text-xs mb-4">通过「添加代理 → Clash 订阅」导入远程订阅</div>
            <el-button type="primary" :icon="Plus" @click="openCreateDialog('clash')">添加订阅</el-button>
          </div>

          <div v-else class="sub-list">
            <div
              v-for="group in subscriptionGroups"
              :key="group.sourceId"
              class="sub-card"
            >
              <div class="sub-card__header">
                <div class="sub-card__info">
                  <div class="sub-card__url" :title="group.sourceUrl">{{ group.sourceUrl }}</div>
                  <div class="sub-card__meta">
                    <span v-if="group.groupName" class="sub-card__tag">{{ group.groupName }}</span>
                    <span class="sub-card__stat">{{ group.count }} 个节点</span>
                    <span class="sub-card__stat">{{ group.healthyCount }} 个可用</span>
                    <span class="sub-card__time">更新于 {{ formatTime(group.lastRefreshAt) }}</span>
                  </div>
                </div>
                <div class="sub-card__actions">
                  <el-button size="small" :icon="View" @click="openSubscriptionDetail(group)">详情</el-button>
                  <el-button
                    size="small"
                    :icon="Refresh"
                    :loading="detailRefreshing && detailSourceId === group.sourceId"
                    @click="handleRefreshSubscription(group)"
                  >
                    更新
                  </el-button>
                  <el-button size="small" type="danger" plain :icon="Delete" @click="handleDeleteSubscription(group)">删除</el-button>
                </div>
              </div>
            </div>
          </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <el-drawer
      v-model="dialogOpen"
      title="添加代理"
      direction="rtl"
      size="min(820px, 92vw)"
      destroy-on-close
      class="proxy-editor-drawer"
    >
      <div class="proxy-editor-drawer__body">
      <div class="proxy-dialog__switch">
        <el-segmented v-model="dialogTab" :options="dialogTabOptions" block />
      </div>

      <div v-if="dialogTab === 'standard'" class="proxy-dialog__body">
        <el-form label-width="96px">
          <el-form-item label="名称" required>
            <el-input v-model="standardForm.name" placeholder="请输入代理名称" />
          </el-form-item>
          <el-form-item label="协议" required>
            <el-select v-model="standardForm.protocol" class="w-full">
              <el-option label="HTTP" value="http" />
              <el-option label="HTTPS" value="https" />
              <el-option label="SOCKS5" value="socks5" />
              <el-option label="SOCKS4" value="socks4" />
            </el-select>
          </el-form-item>
          <el-row :gutter="16">
            <el-col :span="14">
              <el-form-item label="主机" required>
                <el-input v-model="standardForm.host" placeholder="请输入主机地址" />
              </el-form-item>
            </el-col>
            <el-col :span="10">
              <el-form-item label="端口" required>
                <el-input v-model="standardForm.port" placeholder="8080" />
              </el-form-item>
            </el-col>
          </el-row>
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="用户名（可选）">
                <el-input v-model="standardForm.username" placeholder="可选认证信息" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="密码（可选）">
                <el-input v-model="standardForm.password" placeholder="可选认证信息" show-password />
              </el-form-item>
            </el-col>
          </el-row>
          <el-form-item label="DNS（可选）">
            <el-input v-model="standardForm.dnsServers" placeholder="多个 DNS 用逗号分隔" />
          </el-form-item>
          <el-form-item label="分组（可选）">
            <el-input v-model="standardForm.groupName" placeholder="例如：香港 / 美国 / 直连" />
          </el-form-item>
        </el-form>
        <div class="soft-card p-3 mt-4">
          <div class="soft-card__label">预览</div>
          <code class="proxy-preview__code mt-1 block text-xs break-all">{{ buildProxyConfig() || '填写上方信息后自动生成' }}</code>
        </div>
      </div>

      <div v-else-if="dialogTab === 'quick'" class="proxy-dialog__body">
        <div class="flex items-center justify-between mb-3">
          <span class="muted-text">正在寻找合适的代理 IP？</span>
          <el-button link type="primary" @click="handleFetchSubscription">拉取订阅</el-button>
        </div>

        <template v-if="quickForm.mode === 'url' && !quickForm.content">
          <el-form label-width="80px">
            <el-form-item label="订阅 URL">
              <div class="w-full flex gap-2">
                <el-input v-model="quickForm.url" placeholder="https://example.com/sub.yaml" />
                <el-button :loading="fetchingSubscription" @click="handleFetchSubscription">拉取</el-button>
              </div>
            </el-form-item>
          </el-form>
        </template>

        <div v-else>
          <el-input
            v-model="quickForm.content"
            type="textarea"
            :rows="10"
            :placeholder="quickPlaceholder"
          />

          <p class="mt-2 muted-text text-xs">支持 http、https、socks5 协议，格式：协议://[用户名:密码@]主机:端口</p>

          <el-row :gutter="12" class="mt-4">
            <el-col :span="12">
              <el-input v-model="quickForm.groupName" placeholder="分组名称（可选）" size="small">
                <template #prefix>分组</template>
              </el-input>
            </el-col>
            <el-col :span="12">
              <el-input v-model="quickForm.namePrefix" placeholder="名称前缀（可选）" size="small">
                <template #prefix>前缀</template>
              </el-input>
            </el-col>
          </el-row>
        </div>
      </div>

      <div v-else class="proxy-dialog__body">
        <el-form label-width="96px">
          <el-form-item label="订阅 URL" required>
            <div class="w-full flex gap-2">
              <el-input v-model="clashForm.url" placeholder="https://example.com/clash-sub.yaml" />
              <el-button :loading="fetchingSubscription" @click="handleClashFetch">拉取</el-button>
            </div>
          </el-form-item>
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="分组">
                <el-input v-model="clashForm.groupName" placeholder="例如：香港 / 美国 / 直连" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="名称前缀">
                <el-input v-model="clashForm.namePrefix" placeholder="可选" />
              </el-form-item>
            </el-col>
          </el-row>
          <el-form-item label="DNS">
            <el-input v-model="clashForm.dnsServers" placeholder="多个 DNS 用逗号分隔（可选）" />
          </el-form-item>
        </el-form>

        <div v-if="clashForm.fetchedContent" class="subscription-preview">
          <div class="subscription-preview__head">
            <div>
              <div class="subscription-preview__eyebrow">订阅预览</div>
              <div class="subscription-preview__title">已解析远程 Clash 配置</div>
            </div>
            <el-button size="small" :loading="fetchingSubscription" @click="handleClashFetch">
              <el-icon class="mr-1"><Refresh /></el-icon>刷新
            </el-button>
          </div>

          <div class="subscription-preview__metrics">
            <div class="subscription-preview__metric">
              <span>节点数</span>
              <strong>{{ clashForm.fetchedProxyCount }}</strong>
            </div>
            <div class="subscription-preview__metric">
              <span>建议分组</span>
              <strong>{{ clashForm.fetchedSuggestedGroup || clashForm.groupName || '未识别' }}</strong>
            </div>
            <div class="subscription-preview__metric">
              <span>名称前缀</span>
              <strong>{{ clashForm.namePrefix || '不添加' }}</strong>
            </div>
          </div>

          <div v-if="clashForm.fetchedDns" class="subscription-preview__dns">
            <span>DNS</span>
            <code>{{ clashForm.fetchedDns }}</code>
          </div>
        </div>

        <div v-else class="subscription-preview subscription-preview--empty">
          <div class="subscription-preview__empty-title">等待解析订阅</div>
          <div class="subscription-preview__empty-copy">输入订阅 URL 并点击「拉取」，系统会解析 Clash YAML 中的 HTTP / SOCKS / SS / Trojan / VMess 节点。</div>
        </div>
      </div>
      </div>

      <template #footer>
        <el-button @click="dialogOpen = false">取消</el-button>
        <el-button
          v-if="dialogTab === 'standard'"
          type="primary"
          :loading="saving"
          @click="handleSaveStandard"
        >
          创建
        </el-button>
        <el-button
          v-else-if="dialogTab === 'quick'"
          type="primary"
          :loading="importing"
          @click="handleImportQuick"
        >
          导入 {{ quickForm.content ? quickForm.content.split('\n').filter((l) => l.trim()).length : 0 }} 个代理
        </el-button>
        <el-button
          v-else
          type="primary"
          :loading="importing"
          :disabled="!clashForm.fetchedContent"
          @click="handleClashImport"
        >
          导入 {{ clashForm.fetchedProxyCount }} 个节点
        </el-button>
      </template>
    </el-drawer>

    <el-drawer
      v-model="detailOpen"
      title="订阅详情"
      direction="rtl"
      size="min(920px, 92vw)"
      destroy-on-close
      class="proxy-detail-drawer"
    >
      <div class="proxy-detail-drawer__body">
      <div v-loading="detailLoading">
        <el-table :data="detailProxies" max-height="520" empty-text="暂无代理数据">
          <el-table-column prop="proxyName" label="代理名称" min-width="180" show-overflow-tooltip />
          <el-table-column prop="groupName" label="分组" width="120" show-overflow-tooltip />
          <el-table-column prop="proxyConfig" label="配置" min-width="260" show-overflow-tooltip />
          <el-table-column label="绑定实例" min-width="140" show-overflow-tooltip>
            <template #default="{ row }">
              <div v-if="row.boundProfileNames?.length" class="tag-cloud">
                <el-tag v-for="name in row.boundProfileNames.slice(0, 2)" :key="name" effect="plain" size="small">{{ name }}</el-tag>
                <span v-if="row.boundProfileNames.length > 2" class="muted-text">+{{ row.boundProfileNames.length - 2 }}</span>
              </div>
              <span v-else class="muted-text">-</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="90">
            <template #default="{ row }">
              <el-tag :type="proxyHealth(row).type" effect="light" size="small">
                {{ proxyHealth(row).label }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="延迟" width="90">
            <template #default="{ row }">
              <span v-if="row.lastLatencyMs >= 0">{{ row.lastLatencyMs }} ms</span>
              <span v-else class="muted-text">-</span>
            </template>
          </el-table-column>
        </el-table>
      </div>
      </div>

      <template #footer>
        <el-button @click="detailOpen = false">关闭</el-button>
      </template>
    </el-drawer>
  </section>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

:global(.proxy-editor-drawer .el-drawer__body),
:global(.proxy-detail-drawer .el-drawer__body) {
  padding: 0;
  overflow: hidden;
}
:global(.proxy-editor-drawer .el-drawer__footer),
:global(.proxy-detail-drawer .el-drawer__footer) {
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}
.proxy-editor-drawer__body,
.proxy-detail-drawer__body {
  height: 100%;
  overflow: auto;
  padding: 20px;
}
.proxy-dialog__switch {
  margin-bottom: 20px;
}
.proxy-dialog__body {
  min-height: 360px;
}
.proxy-preview__code {
  color: var(--text-soft);
}

.subscription-preview {
  margin-top: 16px;
  padding: 16px;
  @include glass-panel(rgba(255, 255, 255, 0.72));
}

.subscription-preview__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.subscription-preview__eyebrow {
  color: var(--primary-strong);
  font-size: 12px;
  font-weight: 750;
}

.subscription-preview__title {
  margin-top: 4px;
  color: var(--text);
  font-size: 14px;
  font-weight: 750;
}

.subscription-preview__metrics {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-top: 14px;
}

.subscription-preview__metric {
  display: grid;
  gap: 5px;
  min-width: 0;
  padding: 12px;
  border: 1px solid rgba(131, 151, 160, 0.16);
  border-radius: 8px;
  background: rgba(247, 252, 250, 0.78);
}

.subscription-preview__metric span,
.subscription-preview__dns span {
  color: var(--text-soft);
  font-size: 12px;
  font-weight: 650;
}

.subscription-preview__metric strong {
  overflow: hidden;
  color: var(--text);
  font-size: 15px;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.subscription-preview__dns {
  display: grid;
  gap: 6px;
  margin-top: 10px;
}

.subscription-preview__dns code {
  display: block;
  padding: 10px 12px;
  border: 1px solid rgba(131, 151, 160, 0.16);
  border-radius: 8px;
  background: rgba(247, 252, 250, 0.78);
  color: var(--text);
  font-size: 12px;
  line-height: 1.6;
  overflow-wrap: anywhere;
}

.subscription-preview--empty {
  display: grid;
  place-items: center;
  min-height: 132px;
  text-align: center;
}

.subscription-preview__empty-title {
  color: var(--text);
  font-size: 14px;
  font-weight: 750;
}

.subscription-preview__empty-copy {
  max-width: 440px;
  margin-top: 7px;
  color: var(--text-soft);
  font-size: 12px;
  line-height: 1.7;
}

.proxy-content {
  overflow: hidden;
}
.proxy-tabs :deep(.el-tabs__header) {
  margin: 0;
  padding: 0 20px;
  background: rgba(250, 253, 252, 0.72);
  border-bottom: 1px solid var(--border);
  border-radius: 8px 8px 0 0;
}
.proxy-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}
.proxy-tabs :deep(.el-tabs__item) {
  height: 48px;
  line-height: 48px;
  font-size: 14px;
  font-weight: 500;
}
.proxy-tab__body {
  padding: 16px 20px 20px;
}
.proxy-name-cell {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  max-width: 100%;
  min-width: 0;
}
.proxy-name-cell span:first-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sub-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.sub-card {
  padding: 16px;
  @include glass-panel(rgba(255, 255, 255, 0.68), none);
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s;
}
.sub-card:hover {
  border-color: rgba(15, 159, 122, 0.28);
  box-shadow: var(--shadow-subtle);
  transform: translateY(-1px);
}
.sub-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}
.sub-card__info {
  flex: 1;
  min-width: 0;
}
.sub-card__url {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.sub-card__meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
  flex-wrap: wrap;
}
.sub-card__tag {
  display: inline-block;
  padding: 1px 8px;
  font-size: 12px;
  border-radius: 4px;
  background: var(--primary-soft);
  color: var(--primary-strong);
}
.sub-card__stat {
  font-size: 12px;
  color: var(--text-soft);
}
.sub-card__time {
  font-size: 12px;
  color: var(--text-muted);
}
.sub-card__actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

@include respond-down(900px) {
  .proxy-editor-drawer__body,
  .proxy-detail-drawer__body {
    padding: 16px;
  }

  .sub-card__header {
    flex-direction: column;
  }

  .sub-card__actions {
    flex-wrap: wrap;
  }

  .subscription-preview__head {
    flex-direction: column;
  }

  .subscription-preview__metrics {
    grid-template-columns: 1fr;
  }
}
</style>
