<script setup lang="ts">
import type { FingerprintGroupConfig } from '../fingerprint-config'

defineProps<{
  group: FingerprintGroupConfig
}>()

const emit = defineEmits<{
  reset: []
  randomize: []
}>()
</script>

<template>
  <div class="fingerprint-panel-heading">
    <div class="fingerprint-panel-heading__text">
      <span class="fingerprint-panel-title">{{ group.title }}</span>
      <span class="fingerprint-panel-desc">{{ group.description }}</span>
    </div>
    <div class="fingerprint-panel-actions" @click.stop>
      <el-button size="small" text @click="emit('reset')">重置</el-button>
      <el-button size="small" text type="primary" @click="emit('randomize')">随机</el-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.fingerprint-panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  width: 100%;
  min-width: 0;
}
.fingerprint-panel-heading__text {
  display: flex;
  align-items: center;
  min-width: 0;
}
.fingerprint-panel-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 0 0 auto;
}

.fingerprint-panel-actions :deep(.el-button) {
  padding: 4px 8px;
}

.fingerprint-panel-title {
  flex: 0 0 auto;
  color: var(--text);
  font-size: 13px;
  font-weight: 750;
}
.fingerprint-panel-desc {
  margin-left: 10px;
  color: var(--text-soft);
  font-size: 12px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@include respond-down(900px) {
  .fingerprint-panel-heading {
    align-items: flex-start;
    flex-direction: column;
    gap: 8px;
    padding: 8px 0;
  }
  .fingerprint-panel-heading__text {
    flex-wrap: wrap;
  }
  .fingerprint-panel-desc {
    white-space: normal;
  }
}
</style>
