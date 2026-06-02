<script setup lang="ts">
import { Check, Monitor, Refresh } from '@element-plus/icons-vue'
import type { SystemBrowserCandidate } from '@/types'

defineProps<{
  rows: SystemBrowserCandidate[]
  loading: boolean
}>()

const emit = defineEmits<{
  refresh: []
  register: [row: SystemBrowserCandidate]
}>()
</script>

<template>
  <div class="editor-section">
    <div class="section-toolbar">
      <div>
        <div class="section-title section-title--inline">
          <el-icon><Monitor /></el-icon>
          <span>本机 Chrome</span>
        </div>
        <p class="section-copy">
          这里采用延迟检测。只有切换到这个入口，或手动点击"重新检测"时，才会读取系统中的浏览器信息。
        </p>
      </div>
      <el-button type="primary" :icon="Refresh" :loading="loading" @click="emit('refresh')">
        重新检测
      </el-button>
    </div>

    <div v-if="rows.length === 0" class="empty-panel">
      <el-empty description="没有发现可添加的本机 Chrome" />
    </div>

    <div v-else class="table-panel table-panel--dialog">
      <el-table :data="rows" max-height="380px" style="width: 100%">
        <el-table-column prop="name" label="名称" min-width="160" />
        <el-table-column prop="version" label="版本" width="110">
          <template #default="{ row }">
            {{ row.version || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="path" label="路径" min-width="200" show-overflow-tooltip />
        <el-table-column prop="source" label="来源" width="90" />
        <el-table-column label="操作" width="96" fixed="right">
          <template #default="{ row }">
            <el-tag v-if="row.registered" type="success" effect="plain">已添加</el-tag>
            <el-button
              v-else
              size="small"
              type="primary"
              :icon="Check"
              @click="emit('register', row)"
            >
              添加
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.empty-panel {
  display: grid;
  place-items: center;
  min-height: 200px;
  @include glass-panel(var(--surface-soft), none);
}

.table-panel--dialog {
  overflow: auto;
  @include glass-panel;
}

@include respond-down(980px) {
  .section-toolbar {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
