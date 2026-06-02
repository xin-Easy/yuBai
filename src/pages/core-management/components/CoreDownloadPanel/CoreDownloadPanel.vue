<script setup lang="ts">
import { CircleCheck, Download, Refresh } from '@element-plus/icons-vue'
import type { BrowserCoreDownloadOption } from '@/types'
import type { DownloadFormState, DownloadProgressState, DownloadSummary, SegmentedOption } from '@/types/core-management'

defineProps<{
  form: DownloadFormState
  loading: boolean
  downloading: boolean
  options: BrowserCoreDownloadOption[]
  modeOptions: readonly SegmentedOption[]
  progress: DownloadProgressState
  summary: DownloadSummary
}>()

const emit = defineEmits<{
  refreshOptions: []
  selectUrl: []
}>()
</script>

<template>
  <div class="editor-section">
    <div class="download-layout">
      <div class="download-main">
        <div class="section-toolbar compact">
          <div>
            <div class="section-title section-title--inline">
              <el-icon><Download /></el-icon>
              <span>在线下载</span>
            </div>
            <p class="section-copy">
              获取可用版本后，可以直接下载并登记到内核列表。适合新机器初始化或统一版本管理。
            </p>
          </div>
          <el-button :icon="Refresh" :loading="loading" @click="emit('refreshOptions')">
            获取版本
          </el-button>
        </div>

        <div class="editor-panel">
          <el-form label-width="96px">
            <el-form-item label="下载方式">
              <el-segmented v-model="form.mode" :options="modeOptions" block />
            </el-form-item>

            <el-form-item v-if="form.mode === 'official'" label="内核版本" required>
              <el-select
                v-model="form.selectedUrl"
                class="w-full"
                filterable
                placeholder="先获取版本，再选择下载项"
                @change="emit('selectUrl')"
              >
                <el-option
                  v-for="item in options"
                  :key="item.url"
                  :label="`${item.version} / ${item.channel} / ${item.platform}`"
                  :value="item.url"
                />
              </el-select>
            </el-form-item>

            <el-form-item v-else label="下载地址" required>
              <el-input v-model="form.customUrl" placeholder="https://.../chrome.zip" />
            </el-form-item>

            <el-form-item label="内核名称" required>
              <el-input v-model="form.coreName" placeholder="例如：Chrome 136.0.7103.114" />
            </el-form-item>

            <el-form-item label="代理">
              <el-input
                v-model="form.proxyConfig"
                placeholder="可留空，或填写 http://127.0.0.1:7890"
              />
            </el-form-item>

            <el-form-item label="设为默认">
              <el-switch v-model="form.isDefault" />
            </el-form-item>
          </el-form>

          <div v-if="progress" class="progress-panel">
            <el-progress
              class="download-progress"
              :percentage="progress.progress"
              :status="
                progress.phase === 'error'
                  ? 'exception'
                  : progress.phase === 'done'
                    ? 'success'
                    : undefined
              "
            />
            <p class="download-message">{{ progress.message }}</p>
          </div>
        </div>
      </div>

      <aside class="download-aside">
        <div class="aside-title">
          <el-icon><CircleCheck /></el-icon>
          <span>下载摘要</span>
        </div>
        <div class="aside-metrics">
          <div class="metric">
            <span class="metric-label">平台</span>
            <strong class="metric-value">{{ summary.platform }}</strong>
          </div>
          <div class="metric">
            <span class="metric-label">版本</span>
            <strong class="metric-value">{{ summary.version }}</strong>
          </div>
          <div class="metric">
            <span class="metric-label">通道</span>
            <strong class="metric-value">{{ summary.channel }}</strong>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.download-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 260px;
  gap: 20px;
}

.download-main {
  display: grid;
  gap: 0;
}

.download-main .editor-panel {
  @include glass-panel(rgba(255, 255, 255, 0.68));
}

.download-aside {
  display: grid;
  align-content: start;
  gap: 16px;
  padding: 18px;
  @include glass-panel(var(--surface-soft));
}

.aside-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
}

.aside-metrics {
  display: grid;
  gap: 14px;
}

.metric {
  display: grid;
  gap: 4px;
}

.metric-label {
  color: var(--text-soft);
  font-size: 12px;
}

.metric-value {
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.4;
  word-break: break-word;
}

.progress-panel {
  margin-top: 8px;
  border-top: 1px solid var(--border);
  padding-top: 16px;
}

.download-progress {
  margin-top: 4px;
}

.download-message {
  margin: 10px 0 0;
  color: var(--text-soft);
  font-size: 13px;
  line-height: 1.6;
}

@include two-column-stack {
  .section-toolbar {
    flex-direction: column;
    align-items: stretch;
  }

  .download-layout {
    grid-template-columns: 1fr;
  }
}
</style>
