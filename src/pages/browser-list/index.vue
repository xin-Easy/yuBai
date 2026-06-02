<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, QuestionFilled } from '@element-plus/icons-vue'
import ProfileEditorForm from './components/ProfileEditorForm.vue'
import {
  createBrowserProfile,
  deleteBrowserProfile,
  fetchBrowserProfiles,
  startBrowserInstance,
  stopBrowserInstance,
  updateBrowserProfile,
} from '@/api/profiles'
import { fetchBrowserCores } from '@/api/cores'
import { fetchBrowserProxies } from '@/api/proxies'
import { usePageRefresh } from '@/composables/usePageRefresh'
import { defaultFingerprint } from './fingerprint-config'
import type {
  BrowserCore,
  BrowserFingerprint,
  BrowserProfile,
  BrowserProfileInput,
  BrowserProxy,
} from '@/types'

type BrowserProfileFormModel = BrowserProfileInput & {
  fingerprint: BrowserFingerprint
}

const SYSTEM_DIRECT_PROXY_ID = 'proxy-direct'

const loading = ref(false)
const saving = ref(false)
const actionProfileId = ref('')
const createDialogOpen = ref(false)
const editDialogOpen = ref(false)
const editingProfileId = ref('')
const profiles = ref<BrowserProfile[]>([])
const cores = ref<BrowserCore[]>([])
const proxies = ref<BrowserProxy[]>([])
let unlistenBrowserExited: UnlistenFn | null = null

const createForm = reactive<BrowserProfileFormModel>({
  profileName: '',
  userDataDir: '',
  coreId: '',
  fingerprintArgs: [],
  fingerprint: defaultFingerprint(),
  proxyId: '',
  proxyConfig: '',
  launchArgs: ['--disable-sync', '--no-first-run'],
  tags: [],
  keywords: [],
  groupId: '',
})

const editForm = reactive<BrowserProfileFormModel>({
  profileName: '',
  userDataDir: '',
  coreId: '',
  fingerprintArgs: [],
  fingerprint: defaultFingerprint(),
  proxyId: '',
  proxyConfig: '',
  launchArgs: [],
  tags: [],
  keywords: [],
  groupId: '',
})

const summary = computed(() => ({
  total: profiles.value.length,
  running: profiles.value.filter((item) => item.running).length,
  proxyBound: profiles.value.filter((item) => item.proxyId).length,
}))

const coreNameMap = computed(() => new Map(cores.value.map((item) => [item.coreId, item.coreName])))
const proxyNameMap = computed(() => new Map(proxies.value.map((item) => [item.proxyId, item.proxyName])))
const defaultCreateCoreId = computed(() => {
  const defaultCore = cores.value.find((item) => item.isDefault)
  if (defaultCore) return defaultCore.coreId
  return cores.value.length === 1 ? cores.value[0].coreId : ''
})
const defaultCreateProxy = computed(() => proxies.value.find((item) => item.proxyId === SYSTEM_DIRECT_PROXY_ID))
const defaultCreateProxyId = computed(() => defaultCreateProxy.value?.proxyId ?? '')
const defaultCreateProxyConfig = computed(() => defaultCreateProxy.value?.proxyConfig ?? '')
const editingProxyName = computed(() => {
  if (!editForm.proxyId) return ''
  return proxies.value.find((item) => item.proxyId === editForm.proxyId)?.proxyName ?? ''
})

async function loadProfiles() {
  loading.value = true
  try {
    const [profileList, coreList, proxyList] = await Promise.all([
      fetchBrowserProfiles(),
      fetchBrowserCores(),
      fetchBrowserProxies(),
    ])
    profiles.value = profileList
    cores.value = coreList
    proxies.value = proxyList
    if (createDialogOpen.value && !createForm.coreId) {
      createForm.coreId = defaultCreateCoreId.value
    }
    if (createDialogOpen.value && !createForm.proxyId) {
      createForm.proxyId = defaultCreateProxyId.value
      createForm.proxyConfig = defaultCreateProxyConfig.value
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载实例列表失败')
  } finally {
    loading.value = false
  }
}

function resetCreateForm() {
  createForm.profileName = ''
  createForm.userDataDir = ''
  createForm.coreId = defaultCreateCoreId.value
  createForm.fingerprintArgs = []
  createForm.fingerprint = defaultFingerprint()
  createForm.proxyId = defaultCreateProxyId.value
  createForm.proxyConfig = defaultCreateProxyConfig.value
  createForm.launchArgs = ['--disable-sync', '--no-first-run']
  createForm.tags = []
  createForm.keywords = []
  createForm.groupId = ''
}

function handleOpenCreateProfile() {
  editDialogOpen.value = false
  editingProfileId.value = ''
  resetCreateForm()
  createDialogOpen.value = true
}

function resetEditForm() {
  editForm.profileName = ''
  editForm.userDataDir = ''
  editForm.coreId = ''
  editForm.fingerprintArgs = []
  editForm.fingerprint = defaultFingerprint()
  editForm.proxyId = ''
  editForm.proxyConfig = ''
  editForm.launchArgs = []
  editForm.tags = []
  editForm.keywords = []
  editForm.groupId = ''
}

function fillEditForm(row: BrowserProfile) {
  editForm.profileName = row.profileName
  editForm.userDataDir = row.userDataDir
  editForm.coreId = row.coreId
  editForm.fingerprintArgs = [...row.fingerprintArgs]
  editForm.fingerprint = row.fingerprint ? { ...defaultFingerprint(), ...row.fingerprint } : defaultFingerprint()
  editForm.proxyId = row.proxyId
  editForm.proxyConfig = row.proxyConfig
  editForm.launchArgs = [...row.launchArgs]
  editForm.tags = [...row.tags]
  editForm.keywords = [...row.keywords]
  editForm.groupId = row.groupId ?? ''
}

async function handleCreateProfile() {
  if (!createForm.profileName.trim()) {
    ElMessage.warning('请先填写实例名称')
    return
  }

  saving.value = true
  try {
    const created = await createBrowserProfile({
      ...createForm,
      profileName: createForm.profileName.trim(),
      userDataDir: createForm.userDataDir.trim(),
      coreId: createForm.coreId.trim(),
      proxyId: createForm.proxyId.trim(),
      proxyConfig: createForm.proxyConfig.trim(),
      groupId: createForm.groupId?.trim(),
      fingerprint: createForm.fingerprint,
    })
    profiles.value = [created, ...profiles.value]
    createDialogOpen.value = false
    resetCreateForm()
    ElMessage.success('实例已创建')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '创建实例失败')
  } finally {
    saving.value = false
  }
}

function handleOpenEditProfile(row: BrowserProfile) {
  createDialogOpen.value = false
  editingProfileId.value = row.profileId
  fillEditForm(row)
  editDialogOpen.value = true
}

function handleEditDialogClosed() {
  editingProfileId.value = ''
  resetEditForm()
}

async function handleUpdateProfile() {
  if (!editingProfileId.value) {
    return
  }
  if (!editForm.profileName.trim()) {
    ElMessage.warning('请先填写实例名称')
    return
  }

  saving.value = true
  try {
    const updated = await updateBrowserProfile(editingProfileId.value, {
      ...editForm,
      profileName: editForm.profileName.trim(),
      userDataDir: editForm.userDataDir.trim(),
      coreId: editForm.coreId.trim(),
      proxyId: editForm.proxyId.trim(),
      proxyConfig: editForm.proxyConfig.trim(),
      groupId: editForm.groupId?.trim(),
      fingerprint: editForm.fingerprint,
    })
    replaceProfile(updated)
    editDialogOpen.value = false
    ElMessage.success('实例已更新')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '更新实例失败')
  } finally {
    saving.value = false
  }
}

async function handleDeleteProfile(row: BrowserProfile) {
  await ElMessageBox.confirm(`确认删除实例"${row.profileName}"吗？`, '删除确认', {
    type: 'warning',
    confirmButtonText: '删除',
    cancelButtonText: '取消',
  })

  try {
    await deleteBrowserProfile(row.profileId)
    profiles.value = profiles.value.filter((item) => item.profileId !== row.profileId)
    ElMessage.success('实例已删除')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '删除实例失败')
  }
}

function replaceProfile(next: BrowserProfile) {
  profiles.value = profiles.value.map((item) => (item.profileId === next.profileId ? next : item))
}

function runtimeProxyKind(summary: string) {
  const value = (summary || '').trim()
  if (!value || value === '未运行') return 'stopped'
  if (value.startsWith('Mihomo bridge')) return 'mihomo'
  if (value.startsWith('插件 ')) return 'extension'
  if (value.startsWith('直连')) return 'direct'
  return 'unknown'
}

function runtimeProxyLabel(summary: string) {
  switch (runtimeProxyKind(summary)) {
    case 'mihomo':
      return 'Mihomo'
    case 'extension':
      return '插件代理'
    case 'direct':
      return '直连'
    case 'stopped':
      return '未运行'
    default:
      return '未知'
  }
}

function runtimeProxyTagType(summary: string) {
  switch (runtimeProxyKind(summary)) {
    case 'mihomo':
      return 'warning'
    case 'extension':
      return 'success'
    case 'direct':
      return 'info'
    case 'stopped':
      return ''
    default:
      return 'info'
  }
}

function runtimeProxyDetail(summary: string) {
  const value = (summary || '').trim()
  if (!value || value === '未运行') return '实例未启动'
  if (value.startsWith('插件 ')) return value.slice('插件 '.length)
  if (value.startsWith('Mihomo bridge / ')) return value.slice('Mihomo bridge / '.length)
  if (value.startsWith('直连参数 / ')) return value.slice('直连参数 / '.length)
  return value
}

async function listenBrowserExited() {
  unlistenBrowserExited = await listen<BrowserProfile>('browser:instance:exited', (event) => {
    replaceProfile(event.payload)
  })
}

async function handleStartProfile(row: BrowserProfile) {
  actionProfileId.value = row.profileId
  try {
    replaceProfile(await startBrowserInstance(row.profileId))
    ElMessage.success('实例已启动')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '启动实例失败')
  } finally {
    actionProfileId.value = ''
  }
}

async function handleStopProfile(row: BrowserProfile) {
  actionProfileId.value = row.profileId
  try {
    replaceProfile(await stopBrowserInstance(row.profileId))
    ElMessage.success('实例已停止')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '停止实例失败')
  } finally {
    actionProfileId.value = ''
  }
}

onMounted(() => {
  loadProfiles()
  listenBrowserExited().catch((error) => {
    console.warn('failed to listen browser exit event', error)
  })
})
usePageRefresh(loadProfiles)

onUnmounted(() => {
  unlistenBrowserExited?.()
  unlistenBrowserExited = null
})
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">实例列表</h1>
        <p class="page-subtitle">创建、启动和维护浏览器实例，集中查看内核绑定、代理使用和运行状态。</p>
      </div>
      <div class="page-actions">
        <el-button type="primary" :icon="Plus" @click="handleOpenCreateProfile">新建实例</el-button>
      </div>
    </div>

    <div class="page-grid-3 browser-summary">
      <div class="metric-card browser-summary__card">
        <div class="metric-card__label">实例总数</div>
        <div class="metric-card__value">{{ summary.total }}</div>
        <div class="metric-card__text">全部浏览器环境</div>
      </div>
      <div class="metric-card browser-summary__card">
        <div class="metric-card__label">运行中</div>
        <div class="metric-card__value">{{ summary.running }}</div>
        <div class="metric-card__text">正在占用本机资源</div>
      </div>
      <div class="metric-card browser-summary__card">
        <div class="metric-card__label">已绑定代理</div>
        <div class="metric-card__value">{{ summary.proxyBound }}</div>
        <div class="metric-card__text">具备独立网络出口</div>
      </div>
    </div>

    <div class="content-panel table-panel">
      <div class="table-panel__meta">
        <span class="muted-text">{{ profiles.length }} 个实例</span>
        <span class="muted-text">内核 / 代理 / 运行状态</span>
      </div>
      <el-table v-loading="loading" :data="profiles" class="table-panel__table" empty-text="暂无实例">
        <el-table-column prop="profileName" label="实例名称" min-width="190" show-overflow-tooltip />
        <el-table-column label="内核" min-width="160" show-overflow-tooltip>
          <template #default="{ row }">
            {{ coreNameMap.get(row.coreId) || row.coreId || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="代理" min-width="160" show-overflow-tooltip>
          <template #default="{ row }">
            {{ proxyNameMap.get(row.proxyId) || row.proxyId || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="当前代理状态" min-width="190">
          <template #default="{ row }">
            <el-tooltip
              :content="row.runtimeProxySummary || (row.running ? '运行中，状态读取中' : '未运行')"
              placement="top"
            >
              <div class="runtime-proxy">
                <el-tag :type="runtimeProxyTagType(row.runtimeProxySummary)" effect="plain" size="small">
                  {{ runtimeProxyLabel(row.runtimeProxySummary) }}
                </el-tag>
                <span class="runtime-proxy__detail">
                  {{ runtimeProxyDetail(row.runtimeProxySummary || (row.running ? '运行中，状态读取中' : '未运行')) }}
                </span>
              </div>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column label="标签" min-width="150" show-overflow-tooltip>
          <template #default="{ row }">
            <div class="tag-cloud">
              <el-tag v-for="tag in row.tags.slice(0, 3)" :key="tag" effect="plain">{{ tag }}</el-tag>
              <span v-if="row.tags.length === 0" class="muted-text">-</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="110">
          <template #default="{ row }">
            <el-tag :type="row.automationRunId ? 'warning' : row.running ? 'success' : 'info'" effect="plain">
              {{ row.automationRunId ? '自动化占用' : row.running ? '运行中' : '未启动' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="debugPort" width="118">
          <template #header>
            <span class="table-help-title">
              调试端口
              <el-tooltip
                placement="top"
                content="用于 Chrome DevTools Protocol（CDP）连接。实例运行后可用 127.0.0.1:端口 或 ws://127.0.0.1:端口/devtools/browser/... 接入自动化工具。"
              >
                <el-icon class="table-help-title__icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="updatedAt" label="更新时间" min-width="180" show-overflow-tooltip />
        <el-table-column label="操作" width="214" fixed="right">
          <template #default="{ row }">
            <el-button
              v-if="!row.running"
              size="small"
              type="primary"
              plain
              :loading="actionProfileId === row.profileId"
              @click="handleStartProfile(row)"
            >
              启动
            </el-button>
            <el-button
              v-else
              size="small"
              type="warning"
              plain
              :loading="actionProfileId === row.profileId"
              @click="handleStopProfile(row)"
            >
              停止
            </el-button>
            <el-button size="small" plain :disabled="row.running" @click="handleOpenEditProfile(row)">编辑</el-button>
            <el-button size="small" type="danger" plain @click="handleDeleteProfile(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <el-drawer
      v-model="createDialogOpen"
      title="新建实例"
      direction="rtl"
      size="min(960px, 92vw)"
      destroy-on-close
      class="profile-drawer"
    >
      <div class="profile-drawer__body">
        <div class="profile-drawer__form">
          <ProfileEditorForm
            :form="createForm"
            :cores="cores"
            :proxies="proxies"
            mode="create"
          />
        </div>
      </div>

      <template #footer>
        <el-button @click="createDialogOpen = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleCreateProfile">创建</el-button>
      </template>
    </el-drawer>

    <el-drawer
      v-model="editDialogOpen"
      title="编辑实例"
      direction="rtl"
      size="min(960px, 92vw)"
      destroy-on-close
      class="profile-drawer"
      @closed="handleEditDialogClosed"
    >
      <div class="profile-drawer__body">
        <div class="profile-drawer__summary">
          <div>
            <div class="profile-drawer__summary-label">编辑说明</div>
            <div class="profile-drawer__summary-text">运行中的实例不能编辑。修改代理、内核或启动参数后，下次启动该实例时生效。</div>
          </div>
          <div>
            <div class="profile-drawer__summary-label">当前代理</div>
            <div class="profile-drawer__summary-text">{{ editingProxyName || editForm.proxyConfig || '未绑定代理' }}</div>
          </div>
        </div>

        <div class="profile-drawer__form">
          <ProfileEditorForm
            :form="editForm"
            :cores="cores"
            :proxies="proxies"
            mode="edit"
          />
        </div>
      </div>

      <template #footer>
        <el-button @click="editDialogOpen = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleUpdateProfile">保存</el-button>
      </template>
    </el-drawer>
  </section>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

:global(.profile-drawer .el-drawer__body) {
  padding: 0;
  overflow: hidden;
}
:global(.profile-drawer .el-drawer__footer) {
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}
.profile-drawer__body {
  height: 100%;
  overflow: auto;
  padding: 20px;
}
.profile-drawer__form {
  width: 100%;
}
.profile-drawer__summary {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.8fr);
  gap: 12px;
  margin-bottom: 16px;
}
.profile-drawer__summary > div {
  min-width: 0;
  padding: 12px 14px;
  @include glass-panel(var(--surface-soft), none);
}
.profile-drawer__summary-label {
  margin-bottom: 4px;
  color: var(--el-text-color-primary);
  font-size: 13px;
  font-weight: 600;
}
.profile-drawer__summary-text {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  line-height: 1.5;
  overflow-wrap: anywhere;
}
.runtime-proxy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}
.runtime-proxy__detail {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.table-help-title {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.table-help-title__icon {
  color: var(--text-muted);
  cursor: help;
  font-size: 14px;
}

.table-help-title__icon:hover {
  color: var(--primary-strong);
}

@include respond-down(900px) {
  .profile-drawer__summary {
    grid-template-columns: 1fr;
  }

  .profile-drawer__body {
    padding: 16px;
  }
}
</style>
