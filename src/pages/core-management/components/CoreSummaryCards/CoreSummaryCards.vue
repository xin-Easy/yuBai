<script setup lang="ts">
defineProps<{
  total: number
  validCount: number
  defaultCoreName: string
}>()
</script>

<template>
  <div class="page-grid-3">
    <div class="metric-card">
      <div class="metric-card__label">内核数量</div>
      <div class="metric-card__value">{{ total }}</div>
      <div class="metric-card__text">已登记浏览器内核</div>
    </div>
    <div class="metric-card">
      <div class="metric-card__label">路径有效</div>
      <div class="metric-card__value">{{ validCount }}</div>
      <div class="metric-card__text">当前可直接启动</div>
    </div>
    <div class="metric-card">
      <div class="metric-card__label">默认内核</div>
      <div class="metric-card__text">{{ defaultCoreName }}</div>
      <div class="metric-card__text">新实例优先使用</div>
    </div>
  </div>
</template>
