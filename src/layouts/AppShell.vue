<script setup lang="ts">
import AppSidebar from './AppSidebar.vue'
import AppTopbar from './AppTopbar.vue'
</script>

<template>
  <div class="app-shell">
    <AppSidebar />
    <div class="app-shell__body">
      <AppTopbar />
      <main class="app-shell__main">
        <RouterView v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <component :is="Component" />
          </Transition>
        </RouterView>
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;
  display: flex;
  width: 100%;
  height: 100%;
  background:
    radial-gradient(circle at 16% 10%, rgba(113, 211, 185, 0.22), transparent 28%),
    radial-gradient(circle at 92% 4%, rgba(83, 160, 214, 0.16), transparent 30%),
    transparent;
  color: var(--text);
  overflow: hidden;
}

.app-shell::after {
  pointer-events: none;
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255, 255, 255, 0.22) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.2) 1px, transparent 1px);
  background-size: 48px 48px;
  mask-image: linear-gradient(180deg, rgba(0, 0, 0, 0.55), transparent 55%);
  content: "";
}

.app-shell__body {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
  min-height: 0;
}

.app-shell__main {
  flex: 1;
  height: 0;
  min-height: 0;
  overflow-y: auto;
  padding: 20px;
  scroll-behavior: smooth;
}

.page-enter-active,
.page-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.page-enter-from,
.page-leave-to {
  opacity: 0;
  transform: translateY(6px);
}

@media (max-width: 1180px) {
  .app-shell__main {
    padding: 16px;
  }
}

@media (max-width: 760px) {
  .app-shell__main {
    padding: 14px;
  }
}
</style>
