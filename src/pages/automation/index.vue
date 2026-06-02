<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Delete, Edit, Plus, Refresh, SwitchButton, VideoPlay, WarningFilled } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  cancelAutomationRun,
  createAutomationScript,
  deleteAutomationScript,
  fetchAutomationScriptRuns,
  fetchAutomationScripts,
  fetchAutomationState,
  installAutomationRuntime,
  runAutomationScript,
  saveAutomationSettings,
  updateAutomationScript,
} from '@/api/automation'
import { fetchBrowserProfiles } from '@/api/profiles'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { BrowserProfile } from '@/types'
import type {
  AutomationRun,
  AutomationRunEvent,
  AutomationRunLogEvent,
  AutomationScript,
  AutomationState,
} from '@/types/automation'

const loading = ref(false)
const scriptSaving = ref(false)
const scriptDebugging = ref(false)
const running = ref(false)
const runtimeSaving = ref(false)
const runtimeInstalling = ref(false)
const cancellingRunId = ref('')
const scriptDrawerOpen = ref(false)
const runDrawerOpen = ref(false)
const editingScriptId = ref('')
const selectedProfileId = ref('')
const selectedRun = ref<AutomationRun>()
const runningScript = ref<AutomationScript>()
const runTimeoutMs = ref(120000)
const unlisteners: UnlistenFn[] = []
const automationResultPrefix = '__YUBAI_AUTOMATION_RESULT__'
const scriptCodeHelpText = '脚本运行在内置 Node runner 中，已安装 playwright-core 和 typescript。入口默认 main.ts，需 export async function run(ctx)，可使用 ctx.browser、ctx.context、ctx.page、ctx.profileId、ctx.scriptId、ctx.runId、ctx.params、ctx.log，并通过 return 返回结果 JSON。文档：https://playwright.dev/docs/api/class-playwright'

const state = ref<AutomationState>({
  enabled: false,
  runtimeVersion: '',
  headlessDefault: false,
  installed: false,
  ready: false,
  installing: false,
  lastError: '',
  nodeVersion: '',
  playwrightVersion: '',
})
const scripts = ref<AutomationScript[]>([])
const runs = ref<AutomationRun[]>([])
const profiles = ref<BrowserProfile[]>([])

const scriptForm = reactive({
  name: '',
  description: '',
  content: '',
})
const scriptParamRows = ref([{ key: 'url', type: 'string', optional: true }])
const runParamRows = ref([{ key: 'url', value: 'https://example.com' }])

const activeRunCount = computed(() => runs.value.filter((item) => item.status === 'pending' || item.status === 'running').length)
const scriptRuns = computed(() => {
  if (!runningScript.value) {
    return runs.value.slice(0, 8)
  }
  return runs.value.filter((item) => item.scriptId === runningScript.value?.scriptId).slice(0, 8)
})
const selectedProfile = computed(() => profiles.value.find((item) => item.profileId === selectedProfileId.value))
const runtimeStatus = computed(() => {
  if (state.value.ready) return { label: '已就绪', type: 'success' as const }
  if (state.value.installing) return { label: '安装中', type: 'warning' as const }
  return { label: '未就绪', type: 'info' as const }
})

const runStatusMap: Record<AutomationRun['status'], { label: string; type: 'success' | 'warning' | 'danger' | 'info' }> = {
  pending: { label: '等待中', type: 'info' },
  running: { label: '运行中', type: 'warning' },
  success: { label: '成功', type: 'success' },
  failed: { label: '失败', type: 'danger' },
  cancelled: { label: '已取消', type: 'info' },
}

function upsertRun(run: AutomationRun) {
  const index = runs.value.findIndex((item) => item.runId === run.runId)
  if (index >= 0) {
    runs.value.splice(index, 1, run)
  } else {
    runs.value.unshift(run)
  }
  if (selectedRun.value?.runId === run.runId) {
    selectedRun.value = run
  }
}

function firstRunnableProfile(items: BrowserProfile[]) {
  return items.find((profile) => profile.running && profile.debugReady) || items.find((profile) => profile.running) || items[0]
}

function parseRunParams() {
  const params: Record<string, unknown> = {}
  for (const row of runParamRows.value) {
    const key = row.key.trim()
    if (!key && !row.value.trim()) continue
    if (!key) throw new Error('运行参数存在空 key')
    params[key] = parseRunParamValue(row.value)
  }
  return Object.keys(params).length > 0 ? params : undefined
}

function parseRunParamValue(value: string) {
  const raw = value.trim()
  if (!raw) return ''
  try {
    return JSON.parse(raw)
  } catch {
    return value
  }
}

function addRunParamRow() {
  runParamRows.value.push({ key: '', value: '' })
}

function removeRunParamRow(index: number) {
  runParamRows.value.splice(index, 1)
  if (runParamRows.value.length === 0) {
    addRunParamRow()
  }
}

function addScriptParamRow() {
  scriptParamRows.value.push({ key: '', type: 'string', optional: true })
}

function removeScriptParamRow(index: number) {
  scriptParamRows.value.splice(index, 1)
  syncScriptParamsToCode()
}

function syncScriptParamsToCode() {
  scriptForm.content = upsertScriptParamsInterface(scriptForm.content, scriptParamRows.value)
}

function extractScriptParamRows(content: string) {
  const block = content.match(/interface\s+ScriptParams\s*\{([\s\S]*?)\}/)
  if (!block) return [{ key: 'url', type: 'string', optional: true }]
  const rows = block[1]
    .split('\n')
    .map((line) => line.trim().match(/^([A-Za-z_$][\w$]*)(\?)?:\s*([^;]+);?$/))
    .filter((match): match is RegExpMatchArray => Boolean(match))
    .map((match) => ({
      key: match[1],
      optional: match[2] === '?',
      type: match[3].trim(),
    }))
  return rows.length > 0 ? rows : [{ key: 'url', type: 'string', optional: true }]
}

function upsertScriptParamsInterface(content: string, rows: Array<{ key: string; type: string; optional: boolean }>) {
  const fields = rows
    .map((row) => ({
      key: row.key.trim(),
      type: row.type.trim() || 'string',
      optional: row.optional,
    }))
    .filter((row) => row.key)
    .map((row) => `  ${row.key}${row.optional ? '?' : ''}: ${row.type}`)
    .join('\n')
  const nextInterface = `interface ScriptParams {\n${fields || '  // 在上方脚本参数中添加字段'}\n}`
  if (/interface\s+ScriptParams\s*\{[\s\S]*?\}/.test(content)) {
    return content.replace(/interface\s+ScriptParams\s*\{[\s\S]*?\}/, nextInterface)
  }
  return content.replace(/(import[^\n]+\n\n)?/, (prefix) => `${prefix || ''}${nextInterface}\n\n`)
}

function formatRunResultJson(run?: AutomationRun) {
  const raw = run?.resultJson?.trim() || parseRunStdout(run?.stdout).resultPayload
  if (!raw) return ''
  try {
    const payload = JSON.parse(raw)
    const value = payload && typeof payload === 'object' && 'result' in payload ? payload.result : payload
    return JSON.stringify(value, null, 2)
  } catch {
    return raw
  }
}

function formatRunStdout(run?: AutomationRun) {
  return parseRunStdout(run?.stdout).visibleStdout
}

function parseRunStdout(stdout = '') {
  let resultPayload = ''
  const visibleLines = stdout.split('\n').filter((line) => {
    const trimmed = line.trimStart()
    if (trimmed.startsWith(automationResultPrefix)) {
      resultPayload = trimmed.slice(automationResultPrefix.length)
      return false
    }
    return true
  })
  return {
    resultPayload,
    visibleStdout: visibleLines.join('\n').trim(),
  }
}

function shortId(value: string) {
  return value ? value.replace(/^run-/, '').slice(0, 8) : '-'
}

function formatTime(value: string) {
  if (!value) return '-'
  try {
    return new Date(value).toLocaleString('zh-CN')
  } catch {
    return value
  }
}

function latestRunForScript(scriptId: string) {
  return runs.value.find((item) => item.scriptId === scriptId)
}

function isScriptRunning(scriptId: string) {
  return runs.value.some((item) => item.scriptId === scriptId && (item.status === 'pending' || item.status === 'running'))
}

function runStatusType(status?: AutomationRun['status']) {
  return status ? runStatusMap[status]?.type || 'info' : 'info'
}

function runStatusLabel(status?: AutomationRun['status']) {
  return status ? runStatusMap[status]?.label || status : '未运行'
}

async function loadAutomation() {
  loading.value = true
  try {
    const [nextState, nextScripts, nextRuns, nextProfiles] = await Promise.all([
      fetchAutomationState(),
      fetchAutomationScripts(),
      fetchAutomationScriptRuns(),
      fetchBrowserProfiles(),
    ])
    state.value = nextState
    scripts.value = nextScripts
    runs.value = nextRuns
    profiles.value = nextProfiles
    if (!selectedProfileId.value) {
      selectedProfileId.value = firstRunnableProfile(nextProfiles)?.profileId || ''
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载自动化数据失败')
  } finally {
    loading.value = false
  }
}

async function loadProfiles() {
  profiles.value = await fetchBrowserProfiles()
}

function openCreateScript() {
  editingScriptId.value = ''
  scriptForm.name = '示例脚本'
  scriptForm.description = '读取页面标题并返回 JSON'
  scriptParamRows.value = [{ key: 'url', type: 'string', optional: true }]
  scriptForm.content = `import type { Browser, BrowserContext, Page } from 'playwright-core'

interface ScriptParams {
  url?: string
}

interface AutomationContext {
  browser: Browser
  context: BrowserContext
  page: Page
  profileId: string
  scriptId: string
  runId: string
  params: ScriptParams
  log: (message: string, data?: unknown) => void
}

export async function run(ctx: AutomationContext) {
  const page = ctx.page
  const url = ctx.params.url || 'https://example.com'
  ctx.log('opening page', { url, profileId: ctx.profileId, runId: ctx.runId })
  await page.goto(url)
  return {
    url,
    title: await page.title(),
  }
}
`
  scriptDrawerOpen.value = true
}

function openEditScript(script: AutomationScript) {
  editingScriptId.value = script.scriptId
  scriptForm.name = script.name
  scriptForm.description = script.description
  scriptForm.content = script.content
  scriptParamRows.value = extractScriptParamRows(script.content)
  scriptDrawerOpen.value = true
}

function openRunDrawer(script: AutomationScript) {
  runningScript.value = script
  selectedRun.value = runs.value.find((item) => item.scriptId === script.scriptId)
  selectedProfileId.value = selectedProfileId.value || firstRunnableProfile(profiles.value)?.profileId || ''
  runDrawerOpen.value = true
}

function resetScriptForm() {
  editingScriptId.value = ''
  scriptForm.name = ''
  scriptForm.description = ''
  scriptForm.content = ''
  scriptParamRows.value = [{ key: 'url', type: 'string', optional: true }]
}

async function saveScript() {
  if (!scriptForm.name.trim()) {
    ElMessage.warning('请输入脚本名称')
    return undefined
  }
  if (!scriptForm.content.trim()) {
    ElMessage.warning('请输入脚本内容')
    return undefined
  }
  const trimmedName = scriptForm.name.trim()
  const duplicate = scripts.value.find(
    (item) => item.name === trimmedName && item.scriptId !== editingScriptId.value,
  )
  if (duplicate) {
    ElMessage.warning(`脚本名称「${trimmedName}」已存在`)
    return undefined
  }
  syncScriptParamsToCode()

  const input = {
    name: trimmedName,
    description: scriptForm.description.trim(),
    content: scriptForm.content,
  }
  const saved = editingScriptId.value
    ? await updateAutomationScript(editingScriptId.value, input)
    : await createAutomationScript(input)
  const index = scripts.value.findIndex((item) => item.scriptId === saved.scriptId)
  if (index >= 0) {
    scripts.value.splice(index, 1, saved)
  } else {
    scripts.value.unshift(saved)
  }
  return saved
}

async function handleSaveScript() {
  scriptSaving.value = true
  try {
    if (await saveScript()) {
      scriptDrawerOpen.value = false
      resetScriptForm()
      ElMessage.success(editingScriptId.value ? '脚本已更新' : '脚本已创建')
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存脚本失败')
  } finally {
    scriptSaving.value = false
  }
}

async function handleSaveAndDebugScript() {
  scriptDebugging.value = true
  try {
    const saved = await saveScript()
    if (!saved) return
    scriptDrawerOpen.value = false
    resetScriptForm()
    openRunDrawer(saved)
    ElMessage.success('脚本已保存，可提交调试运行')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存脚本失败')
  } finally {
    scriptDebugging.value = false
  }
}

async function handleDeleteScript(script: AutomationScript) {
  await ElMessageBox.confirm(`确定删除脚本「${script.name}」吗？`, '删除脚本', {
    type: 'warning',
    confirmButtonText: '删除',
    cancelButtonText: '取消',
  })
  try {
    await deleteAutomationScript(script.scriptId)
    scripts.value = scripts.value.filter((item) => item.scriptId !== script.scriptId)
    if (runningScript.value?.scriptId === script.scriptId) {
      runDrawerOpen.value = false
      runningScript.value = undefined
    }
    ElMessage.success('脚本已删除')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '删除脚本失败')
  }
}

async function handleRunScript() {
  if (!runningScript.value) {
    ElMessage.warning('请选择脚本')
    return
  }
  if (!selectedProfileId.value) {
    ElMessage.warning('请选择浏览器实例')
    return
  }

  running.value = true
  try {
    const run = await runAutomationScript({
      scriptId: runningScript.value.scriptId,
      profileId: selectedProfileId.value,
      params: parseRunParams(),
      timeoutMs: runTimeoutMs.value,
    })
    upsertRun(run)
    selectedRun.value = run
    ElMessage.success('脚本已加入运行队列')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '脚本执行失败')
  } finally {
    running.value = false
  }
}

async function handleEnableAutomation() {
  runtimeSaving.value = true
  try {
    state.value = await saveAutomationSettings(true, state.value.headlessDefault)
    ElMessage.success('自动化已启用')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '启用自动化失败')
  } finally {
    runtimeSaving.value = false
  }
}

async function handlePrepareRuntime() {
  runtimeInstalling.value = true
  try {
    state.value = await saveAutomationSettings(true, state.value.headlessDefault)
    state.value = await installAutomationRuntime()
    ElMessage.success('自动化运行时已准备')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '准备自动化运行时失败')
  } finally {
    runtimeInstalling.value = false
  }
}

async function handleCancelRun(run: AutomationRun) {
  cancellingRunId.value = run.runId
  try {
    upsertRun(await cancelAutomationRun(run.runId))
    ElMessage.success('运行已取消')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '取消运行失败')
  } finally {
    cancellingRunId.value = ''
  }
}

onMounted(loadAutomation)
onMounted(async () => {
  for (const eventName of ['automation:run:queued', 'automation:run:started', 'automation:run:finished', 'automation:run:failed', 'automation:run:cancelled']) {
    unlisteners.push(
      await listen<AutomationRunEvent>(eventName, (event) => {
        if (event.payload.run) {
          upsertRun(event.payload.run)
          selectedRun.value = event.payload.run
          loadProfiles().catch(() => undefined)
        }
      }),
    )
  }
  unlisteners.push(
    await listen<AutomationRunLogEvent>('automation:run:log', (event) => {
      const run = runs.value.find((item) => item.runId === event.payload.runId)
      if (!run) return
      if (event.payload.stream === 'stderr') {
        run.stderr = [run.stderr, event.payload.line].filter(Boolean).join('\n')
      } else {
        run.stdout = [run.stdout, event.payload.line].filter(Boolean).join('\n')
      }
      upsertRun({ ...run })
    }),
  )
})
onUnmounted(() => {
  for (const unlisten of unlisteners.splice(0)) {
    unlisten()
  }
})
usePageRefresh(loadAutomation)
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">自动化脚本</h1>
        <p class="page-subtitle">脚本由后端 runner 排队执行，通过 CDP 连接指定浏览器实例，并向 run(ctx) 注入页面、上下文和参数。</p>
      </div>
      <div class="page-actions">
        <el-button :icon="Refresh" :loading="loading" @click="loadAutomation">刷新</el-button>
        <el-button type="primary" :icon="Plus" @click="openCreateScript">新建脚本</el-button>
      </div>
    </div>

    <div class="page-grid-4">
      <div class="metric-card">
        <div class="metric-card__label">运行时状态</div>
        <div class="mt-3">
          <el-tag :type="runtimeStatus.type" effect="plain">{{ runtimeStatus.label }}</el-tag>
        </div>
        <div class="metric-card__text">{{ state.lastError || '后端自动化执行环境' }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">脚本数量</div>
        <div class="metric-card__value">{{ scripts.length }}</div>
        <div class="metric-card__text">本地 scripts 目录登记项</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">队列活动</div>
        <div class="metric-card__value">{{ activeRunCount }}</div>
        <div class="metric-card__text">等待中 / 运行中</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">Node / Playwright</div>
        <div class="metric-card__text">{{ state.nodeVersion || '-' }} / {{ state.playwrightVersion || '-' }}</div>
      </div>
    </div>

    <div class="content-panel table-panel automation-table-panel">
      <div class="table-panel__meta">
        <span class="muted-text">{{ scripts.length }} 个脚本</span>
        <span class="muted-text">新脚本入口为 main.ts，兼容旧 main.js；模块需导出 async run(ctx)</span>
      </div>
      <el-table v-loading="loading" :data="scripts" class="table-panel__table" empty-text="暂无自动化脚本">
        <el-table-column prop="name" label="脚本名称" min-width="180" show-overflow-tooltip />
        <el-table-column prop="description" label="说明" min-width="240" show-overflow-tooltip />
        <el-table-column prop="entryFile" label="入口" width="120" />
        <el-table-column prop="version" label="版本" width="90" />
        <el-table-column prop="updatedAt" label="更新时间" min-width="180" show-overflow-tooltip />
        <el-table-column label="最近运行" width="120">
          <template #default="{ row }">
            <el-tag
              v-if="latestRunForScript(row.scriptId)"
              :type="runStatusType(latestRunForScript(row.scriptId)?.status)"
              effect="plain"
            >
              {{ runStatusLabel(latestRunForScript(row.scriptId)?.status) }}
            </el-tag>
            <span v-else class="muted-text">未运行</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="280" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" plain :icon="VideoPlay" :disabled="isScriptRunning(row.scriptId)" @click="openRunDrawer(row)">运行</el-button>
            <el-button size="small" plain :icon="Edit" @click="openEditScript(row)">编辑</el-button>
            <el-button size="small" type="danger" plain :icon="Delete" :disabled="isScriptRunning(row.scriptId)" @click="handleDeleteScript(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <el-drawer
      v-model="runDrawerOpen"
      title="提交自动化运行"
      direction="rtl"
      size="min(920px, 94vw)"
      destroy-on-close
      class="automation-run-drawer"
    >
      <div class="automation-drawer__body">
        <div class="automation-drawer__summary">
          <div>
            <div class="automation-drawer__summary-label">脚本</div>
            <div class="automation-drawer__summary-text">{{ runningScript?.name || '-' }}</div>
          </div>
          <div>
            <div class="automation-drawer__summary-label">入口</div>
            <div class="automation-drawer__summary-text">{{ runningScript?.entryFile || 'main.ts' }}</div>
          </div>
          <div>
            <div class="automation-drawer__summary-label">连接方式</div>
            <div class="automation-drawer__summary-text">chromium.connectOverCDP</div>
          </div>
        </div>

        <el-alert
          v-if="!state.ready"
          :title="state.lastError || '自动化运行时未就绪'"
          type="warning"
          show-icon
          :closable="false"
        >
          <div class="runtime-alert__content">
            <span>运行请求会由后端校验。当前状态未就绪时，请先启用自动化并准备本地运行时。</span>
            <div class="runtime-alert__actions">
              <el-button v-if="!state.enabled" size="small" type="primary" :loading="runtimeSaving" @click="handleEnableAutomation">
                启用自动化
              </el-button>
              <el-button size="small" :loading="runtimeInstalling" @click="handlePrepareRuntime">准备运行时</el-button>
              <el-button size="small" :loading="loading" @click="loadAutomation">刷新状态</el-button>
            </div>
          </div>
        </el-alert>

        <el-form label-width="110px" class="automation-drawer__form">
          <el-form-item label="浏览器实例">
            <el-select v-model="selectedProfileId" class="w-full" placeholder="选择实例">
              <el-option
                v-for="profile in profiles"
                :key="profile.profileId"
                :label="`${profile.profileName}${profile.running ? ' / 运行中' : ''}`"
                :value="profile.profileId"
              >
                <div class="option-row">
                  <span>{{ profile.profileName }}</span>
                  <span class="option-row__meta">{{ profile.running ? `端口 ${profile.debugPort || '-'}` : '后端将启动实例' }}</span>
                </div>
              </el-option>
            </el-select>
          </el-form-item>
          <el-form-item label="实例状态">
            <div class="drawer-soft-card">
              {{
                selectedProfile?.automationRunId
                  ? `自动化占用 / ${shortId(selectedProfile.automationRunId)}`
                  : selectedProfile?.running
                    ? `运行中 / ${selectedProfile.debugPort || '-'}`
                    : '未运行，提交后由后端启动并等待 CDP 端口'
              }}
            </div>
          </el-form-item>
          <el-form-item label="运行参数">
            <div class="param-editor">
              <div v-for="(row, index) in runParamRows" :key="index" class="param-editor__row">
                <el-input v-model="row.key" placeholder="key" />
                <el-input v-model="row.value" placeholder="value，支持 JSON 字面量" />
                <el-button text type="danger" :icon="Delete" @click="removeRunParamRow(index)" />
              </div>
              <el-button size="small" plain :icon="Plus" @click="addRunParamRow">添加参数</el-button>
            </div>
            <div class="form-tip">参数会作为 ctx.params 传入脚本；value 可填写字符串，也可填写 true、123、数组或对象。</div>
          </el-form-item>
          <el-form-item label="超时时间">
            <el-input-number v-model="runTimeoutMs" :min="1000" :max="600000" :step="10000" class="w-full" />
            <div class="form-tip">对应后端 timeoutMs，超时会终止 Node runner。</div>
          </el-form-item>
        </el-form>

        <div class="drawer-section">
          <div class="drawer-section__title">最近运行</div>
          <el-table :data="scriptRuns" size="small" empty-text="暂无运行记录" @row-click="selectedRun = $event">
            <el-table-column label="Run" width="86">
              <template #default="{ row }">
                <code>{{ shortId(row.runId) }}</code>
              </template>
            </el-table-column>
            <el-table-column prop="profileName" label="实例" min-width="120" show-overflow-tooltip />
            <el-table-column label="状态" width="90">
              <template #default="{ row }">
                <el-tag :type="runStatusType(row.status)" effect="plain">
                  {{ runStatusLabel(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="durationMs" label="耗时" width="80" />
            <el-table-column label="操作" width="80">
              <template #default="{ row }">
                <el-button
                  v-if="row.status === 'pending' || row.status === 'running'"
                  link
                  type="danger"
                  :icon="SwitchButton"
                  :loading="cancellingRunId === row.runId"
                  @click.stop="handleCancelRun(row)"
                >
                  取消
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <div class="drawer-section">
          <div class="drawer-section__title">运行输出</div>
          <div v-if="selectedRun" class="run-detail">
            <div class="drawer-soft-grid">
              <div class="drawer-soft-card">
                <span>Run ID</span>
                <strong>{{ selectedRun.runId }}</strong>
              </div>
              <div class="drawer-soft-card">
                <span>开始时间</span>
                <strong>{{ formatTime(selectedRun.startedAt) }}</strong>
              </div>
              <div class="drawer-soft-card">
                <span>错误</span>
                <strong>{{ selectedRun.error || '-' }}</strong>
              </div>
            </div>
            <el-tabs class="mt-3">
              <el-tab-pane label="结果 JSON">
                <el-input
                  :model-value="formatRunResultJson(selectedRun)"
                  type="textarea"
                  :rows="7"
                  readonly
                  placeholder="脚本没有返回结果。请在 run(ctx) 中 return 一个 JSON 可序列化的值。"
                />
              </el-tab-pane>
              <el-tab-pane label="stdout">
                <el-input :model-value="formatRunStdout(selectedRun)" type="textarea" :rows="7" readonly />
              </el-tab-pane>
              <el-tab-pane label="stderr / ctx.log">
                <el-input :model-value="selectedRun.stderr" type="textarea" :rows="7" readonly />
              </el-tab-pane>
            </el-tabs>
          </div>
          <el-empty v-else description="选择运行记录查看输出" />
        </div>
      </div>

      <template #footer>
        <el-button @click="runDrawerOpen = false">关闭</el-button>
        <el-button type="primary" :icon="VideoPlay" :loading="running" @click="handleRunScript">
          提交运行
        </el-button>
      </template>
    </el-drawer>

    <el-drawer
      v-model="scriptDrawerOpen"
      :title="editingScriptId ? '编辑脚本' : '新建脚本'"
      direction="rtl"
      size="min(1180px, 96vw)"
      destroy-on-close
      class="automation-script-drawer"
      @closed="resetScriptForm"
    >
      <div class="script-editor-layout">
        <el-form label-position="top" class="script-editor-layout__meta">
          <div class="script-editor-layout__scroll">
            <el-form-item label="名称">
              <el-input v-model="scriptForm.name" />
            </el-form-item>
            <el-form-item label="说明">
              <el-input v-model="scriptForm.description" />
            </el-form-item>
            <el-form-item label="入口">
              <el-input :model-value="editingScriptId ? '保持原入口' : 'main.ts'" disabled />
              <div class="form-tip">新脚本默认写入 main.ts；历史脚本会继续使用原 entryFile。</div>
            </el-form-item>
            <el-form-item label="脚本参数">
              <div class="param-editor">
                <div v-for="(row, index) in scriptParamRows" :key="index" class="param-editor__row param-editor__row--script">
                  <el-input v-model="row.key" placeholder="字段名" @input="syncScriptParamsToCode" />
                  <el-select v-model="row.type" placeholder="类型" filterable allow-create default-first-option @change="syncScriptParamsToCode">
                    <el-option label="string" value="string" />
                    <el-option label="number" value="number" />
                    <el-option label="boolean" value="boolean" />
                    <el-option label="string[]" value="string[]" />
                    <el-option label="Record<string, unknown>" value="Record<string, unknown>" />
                  </el-select>
                  <el-checkbox v-model="row.optional" @change="syncScriptParamsToCode">可选</el-checkbox>
                  <el-button text type="danger" :icon="Delete" @click="removeScriptParamRow(index)" />
                </div>
                <el-button size="small" plain :icon="Plus" @click="addScriptParamRow">添加参数</el-button>
              </div>
              <div class="form-tip">同步更新 interface ScriptParams；运行抽屉填写的 KV 会作为 ctx.params 传入。</div>
            </el-form-item>
          </div>
        </el-form>

        <el-form label-position="top" class="script-editor-layout__code">
          <el-form-item class="script-editor-code-item">
            <template #label>
              <span class="form-label-with-help">
                代码
                <el-tooltip :content="scriptCodeHelpText" placement="top">
                  <el-icon class="field-help-icon"><WarningFilled /></el-icon>
                </el-tooltip>
              </span>
            </template>
            <div class="code-editor-shell">
              <div class="code-editor-shell__bar">
                <span>main.ts</span>
                <el-tag size="small" effect="plain">TypeScript</el-tag>
              </div>
              <div class="code-editor-shell__body">
                <pre class="code-editor-shell__lines">{{ scriptForm.content.split('\n').map((_, index) => index + 1).join('\n') }}</pre>
                <el-input v-model="scriptForm.content" class="script-code-input" type="textarea" spellcheck="false" />
              </div>
            </div>
          </el-form-item>
        </el-form>
      </div>

      <template #footer>
        <el-button @click="scriptDrawerOpen = false">取消</el-button>
        <el-button :icon="VideoPlay" :loading="scriptDebugging" @click="handleSaveAndDebugScript">保存并调试</el-button>
        <el-button type="primary" :loading="scriptSaving" @click="handleSaveScript">保存</el-button>
      </template>
    </el-drawer>
  </section>
</template>

<style scoped>
:global(.automation-run-drawer .el-drawer__body),
:global(.automation-script-drawer .el-drawer__body) {
  padding: 0;
}

:global(.automation-run-drawer .el-drawer__footer),
:global(.automation-script-drawer .el-drawer__footer) {
  border-top: 1px solid var(--border-subtle);
}

.automation-table-panel {
  min-height: 460px;
}

.automation-drawer__body {
  display: grid;
  gap: 18px;
  padding: 20px;
}

.automation-drawer__summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.automation-drawer__summary > div,
.drawer-soft-card {
  min-width: 0;
  padding: 12px;
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--surface-soft);
}

.automation-drawer__summary-label,
.drawer-soft-card span {
  color: var(--text-soft);
  font-size: 12px;
}

.automation-drawer__summary-text,
.drawer-soft-card strong {
  display: block;
  margin-top: 6px;
  color: var(--text-strong);
  font-size: 13px;
  font-weight: 600;
  word-break: break-all;
}

.automation-drawer__form {
  max-width: none;
}

.script-editor-layout {
  display: grid;
  grid-template-columns: minmax(360px, 0.85fr) minmax(520px, 1.4fr);
  gap: 18px;
  height: calc(100vh - 136px);
  min-height: 560px;
  padding: 20px;
}

.script-editor-layout__meta,
.script-editor-layout__code {
  min-height: 0;
}

.script-editor-layout__meta {
  border-right: 1px solid var(--border-subtle);
  padding-right: 18px;
}

.script-editor-layout__scroll {
  height: 100%;
  overflow: auto;
  padding-right: 4px;
}

.script-editor-layout__code,
.script-editor-code-item,
.script-editor-code-item :deep(.el-form-item__content),
.code-editor-shell {
  height: 100%;
}

.code-editor-shell {
  display: grid;
  grid-template-rows: 38px minmax(0, 1fr);
  width: 100%;
  overflow: hidden;
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--surface-soft);
}

.code-editor-shell__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-subtle);
  color: var(--text-soft);
  font-size: 12px;
}

.code-editor-shell__body {
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr);
  min-height: 0;
}

.code-editor-shell__lines {
  margin: 0;
  padding: 12px 10px;
  overflow: hidden;
  border-right: 1px solid var(--border-subtle);
  color: var(--text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;
  font-size: 13px;
  line-height: 20px;
  text-align: right;
  user-select: none;
}

.script-code-input {
  height: 100%;
}

.script-code-input :deep(.el-textarea__inner) {
  height: 100%;
  min-height: 520px !important;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
  line-height: 20px;
  resize: none;
}

.form-tip {
  width: 100%;
  margin-top: 8px;
  color: var(--text-soft);
  font-size: 12px;
  line-height: 18px;
}

.form-label-with-help {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.field-help-icon {
  color: var(--el-color-warning);
  cursor: help;
  font-size: 14px;
  transform: translateY(1px);
}

.runtime-alert__content {
  display: grid;
  gap: 10px;
}

.runtime-alert__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.param-editor {
  display: grid;
  gap: 8px;
  width: 100%;
}

.param-editor__row {
  display: grid;
  grid-template-columns: minmax(120px, 0.7fr) minmax(180px, 1.3fr) 34px;
  gap: 8px;
  align-items: center;
}

.param-editor__row--script {
  grid-template-columns: minmax(120px, 0.8fr) minmax(160px, 1fr) 70px 34px;
}

.option-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
}

.option-row__meta {
  color: var(--text-soft);
  font-size: 12px;
}

.drawer-section {
  display: grid;
  gap: 12px;
}

.drawer-section__title {
  color: var(--text-strong);
  font-size: 14px;
  font-weight: 700;
}

.drawer-soft-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.run-detail :deep(textarea),
.automation-drawer__form :deep(textarea) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;
}

code {
  color: var(--text-strong);
  font-size: 12px;
}

@media (max-width: 900px) {
  .automation-drawer__summary,
  .drawer-soft-grid {
    grid-template-columns: 1fr;
  }

  .script-editor-layout {
    grid-template-columns: 1fr;
    height: auto;
    min-height: 0;
  }

  .script-editor-layout__meta {
    border-right: 0;
    padding-right: 0;
  }

  .script-editor-layout__scroll {
    height: auto;
    overflow: visible;
  }

  .script-code-editor :deep(.cm-editor) {
    height: 520px;
  }
}
</style>
