<script setup lang="ts">
defineProps<{
  title?: string
  description?: string
}>()
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ title || '功能开发中' }}</h1>
        <p class="page-subtitle">{{ description || '该功能正在开发中，后续版本会逐步补齐。' }}</p>
      </div>
    </div>

    <div class="content-panel feature-stub p-8">
      <el-empty description="功能开发中，敬请期待">
        <el-button type="primary" @click="$router.push('/')">返回控制台</el-button>
      </el-empty>
    </div>
  </section>
</template>

<style scoped>
.feature-stub {
  display: grid;
  min-height: 360px;
  place-items: center;
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.8), rgba(240, 249, 246, 0.84)),
    var(--surface);
}
</style>
