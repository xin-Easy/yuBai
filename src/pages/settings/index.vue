<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Download } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { fetchBrowserSettings, saveBrowserSettings, downloadMihomo } from '@/api/settings'
import {
  fetchAutomationState,
  installAutomationRuntime,
  saveAutomationSettings,
  selfCheckAutomationRuntime,
} from '@/api/automation'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { BrowserSettings } from '@/types'
import type { AutomationRuntimeProgressEvent, AutomationState } from '@/types/automation'

const loading = ref(false)
const saving = ref(false)
const mihomoDownloading = ref(false)
const automationSaving = ref(false)
const automationInstalling = ref(false)
const automationChecking = ref(false)
const runtimeProgress = ref<AutomationRuntimeProgressEvent>()
const unlisteners: UnlistenFn[] = []

const form = reactive<BrowserSettings>({
  defaultFingerprintArgs: [],
  defaultLaunchArgs: [],
  defaultStartUrls: [],
  downloadSource: 'auto',
  mihomoDownloadSource: 'auto',
  proxyMode: 'auto',
  mihomoDownloaded: false,
  mihomoBinaryPath: '',
  restoreLastSession: false,
  startReadyTimeoutMs: 3000,
  startStableWindowMs: 1200,
  userDataRoot: '',
})

const automationState = ref<AutomationState>({
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

const automationRuntimeStatus = computed<{ label: string; type: 'success' | 'warning' | 'info' }>(() => {
  if (automationInstalling.value || automationState.value.installing) {
    return { label: '安装中', type: 'warning' }
  }
  if (!automationState.value.installed) {
    return { label: '未安装', type: 'info' }
  }
  if (!automationState.value.enabled) {
    return { label: '已安装，未启用', type: 'warning' }
  }
  if (automationState.value.ready) {
    return { label: '可运行', type: 'success' }
  }
  return { label: '已安装，需自检', type: 'warning' }
})

const automationRuntimePathText = computed(() => {
  if (!automationState.value.installed) return '尚未安装 Node / Playwright 运行环境'
  return [
    automationState.value.runtimeVersion,
    automationState.value.nodeVersion ? `Node ${automationState.value.nodeVersion}` : '',
    automationState.value.playwrightVersion ? `Playwright ${automationState.value.playwrightVersion}` : '',
  ].filter(Boolean).join(' / ')
})

const automationRuntimeInstallAction = computed(() => automationState.value.installed ? '重新安装运行环境' : '安装运行环境')

async function loadSettings() {
  loading.value = true
  try {
    const [settings, autoState] = await Promise.all([
      fetchBrowserSettings(),
      fetchAutomationState(),
    ])
    Object.assign(form, settings)
    automationState.value = autoState
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载设置失败')
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  saving.value = true
  try {
    await saveBrowserSettings({ ...form })
    ElMessage.success('设置已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存设置失败')
  } finally {
    saving.value = false
  }
}

async function handleMihomoDownload() {
  mihomoDownloading.value = true
  try {
    const path = await downloadMihomo()
    form.mihomoDownloaded = true
    form.mihomoBinaryPath = path
    ElMessage.success('Mihomo 下载完成')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : 'Mihomo 下载失败')
  } finally {
    mihomoDownloading.value = false
  }
}

async function handleSaveAutomationSettings() {
  automationSaving.value = true
  try {
    automationState.value = await saveAutomationSettings(automationState.value.enabled, automationState.value.headlessDefault)
    ElMessage.success('自动化设置已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存自动化设置失败')
  } finally {
    automationSaving.value = false
  }
}

async function handleInstallRuntime() {
  automationInstalling.value = true
  runtimeProgress.value = { phase: 'preparing', message: '准备安装自动化运行环境', progress: 5 }
  try {
    automationState.value = await saveAutomationSettings(automationState.value.enabled, automationState.value.headlessDefault)
    automationState.value = await installAutomationRuntime()
    ElMessage.success('自动化运行环境安装完成')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '自动化运行环境安装失败')
  } finally {
    automationInstalling.value = false
  }
}

async function handleSelfCheck() {
  automationChecking.value = true
  try {
    const result = await selfCheckAutomationRuntime()
    automationState.value = await fetchAutomationState()
    if (result.ok) {
      ElMessage.success(`自检通过：Node ${result.nodeVersion} / Playwright ${result.playwrightVersion}`)
    } else {
      ElMessage.warning(result.error || '自动化运行环境未就绪')
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '自动化运行环境自检失败')
  } finally {
    automationChecking.value = false
  }
}

onMounted(loadSettings)
onMounted(async () => {
  unlisteners.push(
    await listen<AutomationRuntimeProgressEvent>('automation:runtime:progress', (event) => {
      runtimeProgress.value = event.payload
    }),
  )
})
onUnmounted(() => {
  for (const unlisten of unlisteners.splice(0)) {
    unlisten()
  }
})
usePageRefresh(loadSettings)
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">系统设置</h1>
        <p class="page-subtitle">调整默认参数、目录位置和浏览器启动行为。</p>
      </div>
    </div>

    <div class="split-layout split-layout--sidebar">
      <div class="content-panel settings-panel p-5">
        <h2 class="section-title">通用设置</h2>
        <p class="section-copy">修改后点击"保存设置"生效。</p>

        <el-form class="mt-5" label-width="140px">
          <el-form-item label="用户数据目录">
            <el-input v-model="form.userDataRoot" placeholder="例如：data/profiles" />
          </el-form-item>
          <el-form-item label="浏览器下载源">
            <el-select v-model="form.downloadSource" class="w-full">
              <el-option label="自动选择" value="auto" />
              <el-option label="官方源" value="official" />
              <el-option label="NPM Mirror" value="npmmirror" />
            </el-select>
          </el-form-item>
          <el-form-item label="Mihomo 下载源">
            <el-select v-model="form.mihomoDownloadSource" class="w-full">
              <el-option label="自动选择" value="auto" />
              <el-option label="国外源" value="official" />
              <el-option label="国内加速源" value="npmmirror" />
            </el-select>
          </el-form-item>
          <el-form-item label="代理实现方式">
            <el-radio-group v-model="form.proxyMode">
              <el-radio-button label="auto">自动</el-radio-button>
              <el-radio-button label="mihomo">Mihomo</el-radio-button>
              <el-radio-button label="extension">浏览器插件</el-radio-button>
            </el-radio-group>
            <div class="form-tip">
              自动模式下标准代理直连，复杂协议走 Mihomo；Mihomo 模式会让所有代理统一走 Mihomo；插件模式仅支持 HTTP / HTTPS / SOCKS4 / SOCKS5。
            </div>
          </el-form-item>
          <el-divider />

          <h3 class="section-title mb-4">运行环境</h3>
          <p class="section-copy mb-4">管理代理内核和自动化脚本执行环境的本地安装状态。</p>
          <el-form-item label="Mihomo">
            <div class="page-stack w-full gap-2">
              <div class="flex items-center gap-2 w-full">
                <el-tag :type="form.mihomoDownloaded ? 'success' : 'info'" effect="plain">
                  {{ form.mihomoDownloaded ? '已下载' : '未下载' }}
                </el-tag>
                <el-input :model-value="form.mihomoBinaryPath || '尚未生成运行时二进制'" disabled class="flex-1" />
                <el-button
                  :icon="Download"
                  :loading="mihomoDownloading"
                  :type="form.mihomoDownloaded ? 'default' : 'primary'"
                  @click="handleMihomoDownload"
                >
                  {{ form.mihomoDownloaded ? '重新下载' : '立即下载' }}
                </el-button>
              </div>
            </div>
          </el-form-item>
          <el-form-item label="自动化环境">
            <div class="page-stack w-full gap-2">
              <div class="flex items-center gap-2 w-full">
                <el-tag :type="automationRuntimeStatus.type" effect="plain">
                  {{ automationRuntimeStatus.label }}
                </el-tag>
                <el-input :model-value="automationRuntimePathText" disabled class="flex-1" />
                <el-button
                  :icon="Download"
                  :loading="automationInstalling"
                  :type="automationState.installed ? 'default' : 'primary'"
                  @click="handleInstallRuntime"
                >
                  {{ automationRuntimeInstallAction }}
                </el-button>
              </div>
              <el-progress
                v-if="runtimeProgress && automationInstalling"
                :percentage="runtimeProgress.progress"
                :status="runtimeProgress.phase === 'failed' ? 'exception' : undefined"
              />
              <div v-if="runtimeProgress && automationInstalling" class="metric-card__text">{{ runtimeProgress.message }}</div>
            </div>
          </el-form-item>
          <el-form-item label="自动化开关">
            <div class="flex items-center gap-4 w-full">
              <el-switch v-model="automationState.enabled" />
              <span class="form-tip form-tip--inline">关闭后脚本不能提交运行，已安装的运行环境会保留。</span>
            </div>
          </el-form-item>
          <el-form-item label="默认无头模式">
            <el-switch v-model="automationState.headlessDefault" />
          </el-form-item>
          <el-form-item>
            <div class="page-actions">
              <el-button type="primary" :loading="automationSaving" @click="handleSaveAutomationSettings">保存自动化设置</el-button>
              <el-button :loading="automationChecking" @click="handleSelfCheck">自检自动化环境</el-button>
            </div>
          </el-form-item>
          <el-alert
            v-if="automationState.lastError"
            class="mt-2"
            type="warning"
            :title="automationState.lastError"
            show-icon
            :closable="false"
          />
          <el-form-item label="恢复上次会话">
            <el-switch v-model="form.restoreLastSession" />
          </el-form-item>
          <el-form-item label="启动就绪超时">
            <el-input-number v-model="form.startReadyTimeoutMs" :min="500" :step="500" :max="60000" class="w-full" />
          </el-form-item>
          <el-form-item label="稳定窗口时长">
            <el-input-number v-model="form.startStableWindowMs" :min="100" :step="100" :max="10000" class="w-full" />
          </el-form-item>
          <el-form-item label="默认指纹参数">
            <el-select
              v-model="form.defaultFingerprintArgs"
              class="w-full"
              multiple
              filterable
              allow-create
              default-first-option
              placeholder="输入参数后回车"
            />
          </el-form-item>
          <el-form-item label="默认启动参数">
            <el-select
              v-model="form.defaultLaunchArgs"
              class="w-full"
              multiple
              filterable
              allow-create
              default-first-option
              placeholder="输入参数后回车"
            />
          </el-form-item>
          <el-form-item label="默认启动网址">
            <el-select
              v-model="form.defaultStartUrls"
              class="w-full"
              multiple
              filterable
              allow-create
              default-first-option
              placeholder="输入网址后回车"
            />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" :loading="saving" @click="handleSave">保存设置</el-button>
          </el-form-item>
        </el-form>
      </div>

      <div class="page-stack">
        <div class="content-panel settings-panel p-5">
          <h2 class="section-title">说明</h2>
          <p class="section-copy">这里展示的是当前后端实际支持的浏览器运行配置。修改用户数据目录后，通常需要重启应用生效。</p>
        </div>

        <div class="content-panel settings-panel p-5">
          <h2 class="section-title">当前配置</h2>
          <div class="page-stack">
            <div class="soft-card p-4">
              <div class="soft-card__label">下载源</div>
              <div class="soft-card__text">{{ form.downloadSource }}</div>
            </div>
            <div class="soft-card p-4">
              <div class="soft-card__label">Mihomo</div>
              <div class="soft-card__text">{{ form.mihomoDownloaded ? '已下载' : '未下载' }} / {{ form.mihomoDownloadSource }}</div>
            </div>
            <div class="soft-card p-4">
              <div class="soft-card__label">自动化环境</div>
              <div class="soft-card__text">{{ automationRuntimeStatus.label }}</div>
            </div>
            <div class="soft-card p-4">
              <div class="soft-card__label">代理实现</div>
              <div class="soft-card__text">
                {{ form.proxyMode === 'extension' ? '浏览器插件' : form.proxyMode === 'mihomo' ? 'Mihomo' : '自动' }}
              </div>
            </div>
            <div class="soft-card p-4">
              <div class="soft-card__label">恢复上次会话</div>
              <div class="soft-card__text">{{ form.restoreLastSession ? '是' : '否' }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.form-tip {
  width: 100%;
  margin-top: 8px;
  color: var(--text-soft);
  font-size: 12px;
  line-height: 18px;
}

.form-tip--inline {
  width: auto;
  margin-top: 0;
}

.settings-panel :deep(.el-form) {
  max-width: 860px;
}

.settings-panel :deep(.el-form-item) {
  margin-bottom: 19px;
}

@media (max-width: 760px) {
  .settings-panel :deep(.el-form) {
    max-width: none;
  }

  .settings-panel :deep(.el-form-item__label) {
    width: 100% !important;
    justify-content: flex-start;
    margin-bottom: 6px;
  }

  .settings-panel :deep(.el-form-item__content) {
    margin-left: 0 !important;
  }
}
</style>
