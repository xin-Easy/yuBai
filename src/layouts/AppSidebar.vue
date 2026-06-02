<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Fold, Expand } from '@element-plus/icons-vue'
import { navigationConfig, projectConfig } from '@/router/navigation'
import { useLayoutStore } from '@/store/layout'

const route = useRoute()
const router = useRouter()
const layout = useLayoutStore()

const sidebarWidth = computed(() => (layout.sidebarCollapsed ? '64px' : '220px'))

function isActive(path: string) {
  return route.path === path || (path !== '/' && route.path.startsWith(`${path}/`))
}
</script>

<template>
  <aside class="sidebar-shell transition-all duration-200" :style="{ width: sidebarWidth }">
    <div class="sidebar-brand">
      <div class="sidebar-brand__logo">{{ projectConfig.shortName }}</div>
      <div v-if="!layout.sidebarCollapsed" class="sidebar-brand__copy">
        <div class="sidebar-brand__title">{{ projectConfig.name }}</div>
        <div class="sidebar-brand__desc">{{ projectConfig.description }}</div>
      </div>
    </div>

    <nav class="sidebar-nav">
      <section v-for="section in navigationConfig" :key="section.title" class="sidebar-section">
        <div v-if="!layout.sidebarCollapsed" class="sidebar-section__title">{{ section.title }}</div>
        <button
          v-for="item in section.items"
          :key="item.path"
          class="sidebar-link"
          :class="{ 'is-active': isActive(item.path) }"
          :title="layout.sidebarCollapsed ? item.name : undefined"
          @click="router.push(item.path)"
        >
          <el-icon :size="18">
            <component :is="item.icon" />
          </el-icon>
          <span v-if="!layout.sidebarCollapsed" class="sidebar-link__label">{{ item.name }}</span>
        </button>
      </section>
    </nav>

    <div class="sidebar-footer">
      <el-button class="sidebar-footer__button" :icon="layout.sidebarCollapsed ? Expand : Fold" @click="layout.toggleSidebar">
        <span v-if="!layout.sidebarCollapsed">收起侧栏</span>
      </el-button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar-shell {
  position: relative;
  z-index: 2;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  height: 100%;
  border-right: 1px solid rgba(131, 151, 160, 0.2);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.82) 0%, rgba(245, 251, 250, 0.7) 100%);
  box-shadow: 18px 0 42px rgba(30, 61, 72, 0.05);
  backdrop-filter: blur(18px);
}

.sidebar-brand {
  display: flex;
  gap: 10px;
  align-items: center;
  height: var(--app-chrome-height);
  min-height: var(--app-chrome-height);
  padding: 0 16px;
  border-bottom: 1px solid rgba(131, 151, 160, 0.16);
}

.sidebar-brand__logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 8px;
  background: linear-gradient(135deg, #0f9f7a 0%, #69cfb6 100%);
  color: #ffffff;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
  box-shadow: 0 12px 24px rgba(15, 159, 122, 0.22);
}

.sidebar-brand__copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.sidebar-brand__title {
  color: var(--text);
  font-size: 14px;
  font-weight: 750;
}

.sidebar-brand__desc {
  overflow: hidden;
  color: var(--text-soft);
  font-size: 11px;
  line-height: 1.5;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-nav {
  overflow-y: auto;
  padding: 12px 8px;
}

.sidebar-section {
  display: grid;
  gap: 6px;
  margin-bottom: 18px;
}

.sidebar-section__title {
  padding: 0 10px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 750;
  letter-spacing: 0;
}

.sidebar-link {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  height: 42px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: #526b76;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.16s ease, color 0.16s ease, transform 0.16s ease;
}

.sidebar-link:hover {
  background: rgba(235, 249, 244, 0.9);
  color: var(--primary-strong);
}

.sidebar-link.is-active {
  background: rgba(224, 248, 238, 0.96);
  color: var(--primary-strong);
  transform: translateX(1px);
}

.sidebar-link.is-active::before {
  position: absolute;
  left: 0;
  top: 10px;
  width: 3px;
  height: 22px;
  border-radius: 99px;
  background: var(--primary);
  content: "";
}

.sidebar-link__label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-footer {
  padding: 12px 10px 14px;
  border-top: 1px solid rgba(131, 151, 160, 0.16);
}

.sidebar-footer__button {
  width: 100%;
}
</style>
