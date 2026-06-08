<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Bell, Refresh, Search, Setting } from '@element-plus/icons-vue'
import { requestPageRefresh } from '@/composables/usePageRefresh'

const route = useRoute()
const router = useRouter()

const title = computed(() => {
  if (typeof route.meta.title === 'string') {
    return route.meta.title
  }
  if (route.path === '/') {
    return '控制台'
  }
  if (route.path === '/browser/list') {
    return '实例列表'
  }
  return 'yubai'
})

const subtitle = computed(() => {
  if (typeof route.meta.description === 'string') {
    return route.meta.description
  }
  return '浏览器实例、代理、内核和自动化能力的统一工作台'
})
</script>

<template>
  <header class="topbar-shell">
    <div class="topbar-copy" :title="`${title}\n${subtitle}`">
      <div class="topbar-title">{{ title }}</div>
      <div class="topbar-subtitle">{{ subtitle }}</div>
    </div>

    <div class="topbar-search">
      <el-input placeholder="搜索实例、代理、内核或页面" :prefix-icon="Search" clearable />
    </div>

    <div class="topbar-actions">
      <el-tooltip content="刷新当前页面" placement="bottom">
        <el-button :icon="Refresh" circle @click="requestPageRefresh" />
      </el-tooltip>
      <el-tooltip content="通知" placement="bottom">
        <el-button :icon="Bell" circle />
      </el-tooltip>
      <el-tooltip content="系统设置" placement="bottom">
        <el-button :icon="Setting" circle @click="router.push('/settings')" />
      </el-tooltip>
    </div>
  </header>
</template>

<style scoped>
.topbar-shell {
  position: relative;
  z-index: 2;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(220px, 360px) auto;
  align-items: center;
  gap: 16px;
  height: var(--app-chrome-height);
  min-height: var(--app-chrome-height);
  padding: 0 22px;
  border-bottom: 1px solid rgba(131, 151, 160, 0.18);
  background: rgba(255, 255, 255, 0.68);
  box-shadow: 0 12px 32px rgba(30, 61, 72, 0.04);
  backdrop-filter: blur(18px);
}

.topbar-copy {
  min-width: 0;
  overflow: hidden;
}

.topbar-title {
  overflow: hidden;
  color: var(--text);
  font-size: 15px;
  font-weight: 750;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar-subtitle {
  overflow: hidden;
  color: var(--text-soft);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar-search {
  min-width: 0;
  width: 100%;
}

.topbar-search :deep(.el-input__wrapper) {
  height: 36px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
}

.topbar-actions {
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
  gap: 8px;
}

@media (max-width: 1180px) {
  .topbar-shell {
    grid-template-columns: minmax(0, 1fr) minmax(180px, 280px) auto;
    gap: 12px;
    padding: 0 16px;
  }

  .topbar-subtitle {
    display: none;
  }
}

@media (max-width: 960px) {
  .topbar-shell {
    grid-template-columns: 1fr auto;
    height: auto;
    min-height: var(--app-chrome-height);
    padding-top: 12px;
  }

  .topbar-search {
    grid-column: 1 / -1;
    order: 3;
    padding-bottom: 12px;
  }
}
</style>
