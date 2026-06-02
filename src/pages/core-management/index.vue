<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Download, Plus, Search } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePageRefresh } from '@/composables/usePageRefresh'
import {
  deleteBrowserCore,
  detectSystemBrowsers,
  downloadBrowserCore,
  fetchBrowserCoreDownloadOptions,
  fetchBrowserCoreExtendedInfo,
  fetchBrowserCores,
  registerSystemBrowser,
  saveBrowserCore,
  scanLocalBrowserCores,
  validateBrowserCore,
} from '@/api/cores'
import type {
  BrowserCore,
  BrowserCoreDownloadOption,
  BrowserCoreDownloadProgress,
  BrowserCoreExtendedInfo,
  BrowserCoreInput,
  BrowserCoreValidateResult,
  SystemBrowserCandidate,
} from '@/types'
import { CoreDownloadPanel } from './components/CoreDownloadPanel/index.ts'
import { CoreManualEditor } from './components/CoreManualEditor/index.ts'
import { CoreSummaryCards } from './components/CoreSummaryCards/index.ts'
import { CoreSystemBrowserPanel } from './components/CoreSystemBrowserPanel/index.ts'
import { CoreTable } from './components/CoreTable/index.ts'
import type {
  CoreRow,
  DownloadMode,
  EditorMode,
  EditorSource,
} from '@/types/core-management'

const loading = ref(false)
const saving = ref(false)
const scanning = ref(false)
const detecting = ref(false)
const downloadOptionsLoading = ref(false)
const downloading = ref(false)

const cores = ref<BrowserCore[]>([])
const extendedInfo = ref<BrowserCoreExtendedInfo[]>([])
const systemBrowsers = ref<SystemBrowserCandidate[]>([])
const systemBrowsersLoaded = ref(false)
const downloadOptions = ref<BrowserCoreDownloadOption[]>([])
const downloadProgress = ref<BrowserCoreDownloadProgress | null>(null)

const editorOpen = ref(false)
const editorMode = ref<EditorMode>('create')
const editorSource = ref<EditorSource>('manual')
const pathValidation = ref<BrowserCoreValidateResult | null>(null)

const sourceOptions = [
  { label: '手动添加', value: 'manual' },
  { label: '本机 Chrome', value: 'system' },
  { label: '在线下载', value: 'download' },
] as const

const downloadModeOptions = [
  { label: '官方版本', value: 'official' },
  { label: '自定义地址', value: 'custom' },
] as const

const form = reactive<BrowserCoreInput>({
  coreId: '',
  coreName: '',
  corePath: '',
  isDefault: false,
})

const downloadForm = reactive({
  mode: 'official' as DownloadMode,
  selectedUrl: '',
  customUrl: '',
  coreName: '',
  proxyConfig: '',
  isDefault: false,
})

let unlistenDownload: UnlistenFn | null = null

const rows = computed<CoreRow[]>(() => {
  const infoMap = new Map(extendedInfo.value.map((item) => [item.coreId, item]))
  return cores.value.map((core) => {
    const info = infoMap.get(core.coreId)
    return {
      ...core,
      chromeVersion: info?.chromeVersion ?? '',
      instanceCount: info?.instanceCount ?? 0,
      pathValid: info?.pathValid ?? true,
      pathMessage: info?.pathMessage ?? '',
    }
  })
})

const dialogTitle = computed(() => (editorMode.value === 'edit' ? '编辑内核' : '添加内核'))
const validCoreCount = computed(() => rows.value.filter((item) => item.pathValid).length)
const defaultCoreName = computed(() => rows.value.find((item) => item.isDefault)?.coreName || '-')
const shouldDefaultNewCore = computed(() => cores.value.length === 0)

const selectedDownload = computed(() =>
  downloadOptions.value.find((item) => item.url === downloadForm.selectedUrl),
)

const downloadSummary = computed(() => {
  if (downloadForm.mode === 'custom') {
    return {
      platform: '自定义',
      version: '自定义',
      channel: '手动输入',
    }
  }

  return {
    platform: selectedDownload.value?.platform || '-',
    version: selectedDownload.value?.version || '-',
    channel: selectedDownload.value?.channel || '-',
  }
})

watch(editorSource, async (next) => {
  if (!editorOpen.value || editorMode.value === 'edit') {
    return
  }

  if (next === 'system' && !systemBrowsersLoaded.value && !detecting.value) {
    await ensureSystemBrowsersLoaded()
  }

  if (next === 'download' && downloadOptions.value.length === 0) {
    await loadDownloadOptions()
  }
})

function messageOf(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function resetManualForm() {
  form.coreId = ''
  form.coreName = ''
  form.corePath = ''
  form.isDefault = shouldDefaultNewCore.value
  pathValidation.value = null
}

function resetDownloadForm() {
  downloadForm.mode = 'official'
  downloadForm.selectedUrl = ''
  downloadForm.customUrl = ''
  downloadForm.coreName = ''
  downloadForm.proxyConfig = ''
  downloadForm.isDefault = shouldDefaultNewCore.value
  downloadProgress.value = null
}

function resetCreateForms() {
  resetManualForm()
  resetDownloadForm()
}

async function loadCores() {
  loading.value = true
  try {
    const [coreList, infoList] = await Promise.all([
      fetchBrowserCores(),
      fetchBrowserCoreExtendedInfo(),
    ])
    cores.value = coreList
    extendedInfo.value = infoList
  } catch (error) {
    ElMessage.error(messageOf(error, '加载内核列表失败'))
  } finally {
    loading.value = false
  }
}

function openCreateDialog() {
  editorMode.value = 'create'
  editorSource.value = 'manual'
  resetCreateForms()
  editorOpen.value = true
}

function openEditDialog(row: BrowserCore) {
  editorMode.value = 'edit'
  editorSource.value = 'manual'
  form.coreId = row.coreId
  form.coreName = row.coreName
  form.corePath = row.corePath
  form.isDefault = row.isDefault
  pathValidation.value = null
  editorOpen.value = true
}

async function handleValidatePath() {
  const path = form.corePath.trim()
  if (!path) {
    ElMessage.warning('请先填写内核路径')
    return
  }

  try {
    pathValidation.value = await validateBrowserCore(path)
    if (pathValidation.value.valid) {
      ElMessage.success('路径校验通过')
    } else {
      ElMessage.warning(pathValidation.value.message)
    }
  } catch (error) {
    ElMessage.error(messageOf(error, '路径校验失败'))
  }
}

async function handleSaveCore() {
  if (!form.coreName.trim() || !form.corePath.trim()) {
    ElMessage.warning('请填写内核名称和路径')
    return
  }

  saving.value = true
  try {
    await saveBrowserCore({
      coreId: form.coreId?.trim(),
      coreName: form.coreName.trim(),
      corePath: form.corePath.trim(),
      isDefault: form.isDefault,
    })
    editorOpen.value = false
    await loadCores()
    ElMessage.success(editorMode.value === 'edit' ? '内核已更新' : '内核已添加')
  } catch (error) {
    ElMessage.error(messageOf(error, '保存内核失败'))
  } finally {
    saving.value = false
  }
}

async function handleSetDefault(row: BrowserCore) {
  try {
    await saveBrowserCore({ ...row, isDefault: true })
    await loadCores()
    ElMessage.success('已设为默认内核')
  } catch (error) {
    ElMessage.error(messageOf(error, '设置默认内核失败'))
  }
}

async function handleDeleteCore(row: CoreRow) {
  if (row.isDefault) {
    ElMessage.warning('默认内核不能删除，请先将其他内核设为默认')
    return
  }

  await ElMessageBox.confirm(`确认删除内核"${row.coreName}"吗？`, '删除确认', {
    type: 'warning',
    confirmButtonText: '删除',
    cancelButtonText: '取消',
  })

  try {
    await deleteBrowserCore(row.coreId)
    await loadCores()
    ElMessage.success('内核已删除')
  } catch (error) {
    ElMessage.error(messageOf(error, '删除内核失败'))
  }
}

async function handleScanLocalCores() {
  scanning.value = true
  try {
    cores.value = await scanLocalBrowserCores()
    extendedInfo.value = await fetchBrowserCoreExtendedInfo()
    ElMessage.success('本地内核目录扫描完成')
  } catch (error) {
    ElMessage.error(messageOf(error, '扫描本地内核失败'))
  } finally {
    scanning.value = false
  }
}

async function ensureSystemBrowsersLoaded(force = false) {
  if (!force && systemBrowsersLoaded.value) {
    return
  }

  detecting.value = true
  try {
    systemBrowsers.value = await detectSystemBrowsers()
    systemBrowsersLoaded.value = true
    if (systemBrowsers.value.length === 0) {
      ElMessage.info('没有发现本机 Chrome')
    }
  } catch (error) {
    ElMessage.error(messageOf(error, '检测本机 Chrome 失败'))
  } finally {
    detecting.value = false
  }
}

async function handleRegisterSystemBrowser(row: SystemBrowserCandidate) {
  try {
    const defaultName = row.version ? `${row.name} ${row.version}` : row.name
    await registerSystemBrowser(row.path, defaultName, shouldDefaultNewCore.value)
    row.registered = true
    editorOpen.value = false
    await loadCores()
    ElMessage.success('本机 Chrome 已添加')
  } catch (error) {
    ElMessage.error(messageOf(error, '添加本机 Chrome 失败'))
  }
}

async function loadDownloadOptions() {
  downloadOptionsLoading.value = true
  try {
    downloadOptions.value = await fetchBrowserCoreDownloadOptions(28)
    const first = downloadOptions.value[0]
    if (first && !downloadForm.selectedUrl) {
      downloadForm.selectedUrl = first.url
      if (!downloadForm.coreName.trim()) {
        downloadForm.coreName = `Chrome ${first.version}`
      }
    }
  } catch (error) {
    ElMessage.error(messageOf(error, '获取下载版本失败'))
  } finally {
    downloadOptionsLoading.value = false
  }
}

function handleSelectDownloadUrl() {
  const item = selectedDownload.value
  if (item) {
    downloadForm.coreName = `Chrome ${item.version}`
  }
}

async function handleStartDownload() {
  const url = downloadForm.mode === 'official' ? downloadForm.selectedUrl : downloadForm.customUrl
  const version = downloadForm.mode === 'official' ? selectedDownload.value?.version || 'unknown' : 'custom'

  if (!downloadForm.coreName.trim() || !url.trim()) {
    ElMessage.warning('请填写内核名称，并选择下载地址')
    return
  }

  downloading.value = true
  downloadProgress.value = {
    phase: 'starting',
    progress: 0,
    message: '准备下载',
  }

  try {
    await downloadBrowserCore({
      coreName: downloadForm.coreName.trim(),
      version,
      url: url.trim(),
      proxyConfig: downloadForm.proxyConfig.trim(),
      isDefault: downloadForm.isDefault || shouldDefaultNewCore.value,
    })
    ElMessage.success('下载任务已启动')
  } catch (error) {
    downloading.value = false
    downloadProgress.value = null
    ElMessage.error(messageOf(error, '启动下载失败'))
  }
}

async function setupDownloadListener() {
  try {
    unlistenDownload = await listen<BrowserCoreDownloadProgress>(
      'browser-core:download-progress',
      async (event) => {
        downloadProgress.value = event.payload
        if (event.payload.phase === 'done') {
          downloading.value = false
          editorOpen.value = false
          await loadCores()
          ElMessage.success(event.payload.message || '内核下载完成')
        } else if (event.payload.phase === 'error') {
          downloading.value = false
          ElMessage.error(event.payload.message || '内核下载失败')
        }
      },
    )
  } catch {
    unlistenDownload = null
  }
}

onMounted(async () => {
  await setupDownloadListener()
  await loadCores()
})
usePageRefresh(loadCores)

onBeforeUnmount(() => {
  if (unlistenDownload) {
    unlistenDownload()
  }
})
</script>

<template>
  <section class="core-page">
    <div class="page-header">
      <div>
        <h1 class="page-title">内核管理</h1>
        <p class="page-subtitle">
          页面进入时只加载已登记内核，不主动检测本机浏览器。切换到"本机 Chrome"或手动点击重新检测时，才执行系统探测。
        </p>
      </div>
      <div class="page-actions">
        <el-button :icon="Search" :loading="scanning" @click="handleScanLocalCores">
          扫描目录
        </el-button>
        <el-button type="primary" :icon="Plus" @click="openCreateDialog">添加内核</el-button>
      </div>
    </div>

    <CoreSummaryCards
      :total="rows.length"
      :valid-count="validCoreCount"
      :default-core-name="defaultCoreName"
    />

    <CoreTable
      :rows="rows"
      :loading="loading"
      @edit="openEditDialog"
      @set-default="handleSetDefault"
      @delete="handleDeleteCore"
    />

    <el-drawer
      v-model="editorOpen"
      :title="dialogTitle"
      direction="rtl"
      size="min(900px, 92vw)"
      destroy-on-close
      class="core-editor"
    >
      <div class="core-editor__body">
      <div class="editor-shell">
        <div v-if="editorMode === 'create'" class="source-switch">
          <el-segmented v-model="editorSource" :options="sourceOptions" block />
        </div>

        <CoreManualEditor
          v-if="editorSource === 'manual'"
          :form="form"
          :path-validation="pathValidation"
          @validate="handleValidatePath"
        />

        <CoreSystemBrowserPanel
          v-else-if="editorSource === 'system'"
          :rows="systemBrowsers"
          :loading="detecting"
          @refresh="ensureSystemBrowsersLoaded(true)"
          @register="handleRegisterSystemBrowser"
        />

        <CoreDownloadPanel
          v-else
          :form="downloadForm"
          :loading="downloadOptionsLoading"
          :downloading="downloading"
          :options="downloadOptions"
          :mode-options="downloadModeOptions"
          :progress="downloadProgress"
          :summary="downloadSummary"
          @refresh-options="loadDownloadOptions"
          @select-url="handleSelectDownloadUrl"
        />
      </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="editorOpen = false">取消</el-button>
          <el-button
            v-if="editorSource === 'manual'"
            type="primary"
            :loading="saving"
            @click="handleSaveCore"
          >
            {{ editorMode === 'edit' ? '保存' : '添加' }}
          </el-button>
          <el-button
            v-else-if="editorSource === 'download'"
            type="primary"
            :icon="Download"
            :loading="downloading"
            @click="handleStartDownload"
          >
            开始下载
          </el-button>
        </div>
      </template>
    </el-drawer>
  </section>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.core-page {
  display: grid;
  gap: 18px;
}

:global(.core-editor .el-drawer__body) {
  padding: 0;
  overflow: hidden;
}
:global(.core-editor .el-drawer__footer) {
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}
.core-editor__body {
  height: 100%;
  overflow: auto;
  padding: 20px;
}
.editor-shell {
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 480px;
}

.source-switch {
  padding: 4px;
  @include glass-panel(rgba(247, 252, 250, 0.72), none);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

@include respond-down(900px) {
  .core-editor__body {
    padding: 16px;
  }
}
</style>
