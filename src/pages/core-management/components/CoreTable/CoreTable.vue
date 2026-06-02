<script setup lang="ts">
import { Edit } from '@element-plus/icons-vue'
import type { BrowserCore } from '@/types'
import type { CoreRow } from '@/types/core-management'

defineProps<{
  rows: CoreRow[]
  loading: boolean
}>()

const emit = defineEmits<{
  edit: [row: BrowserCore]
  setDefault: [row: BrowserCore]
  delete: [row: CoreRow]
}>()
</script>

<template>
  <div class="content-panel table-panel">
    <div class="table-panel__meta">
      <span class="muted-text">{{ rows.length }} 个内核</span>
      <span class="muted-text">版本 / 路径 / 默认状态</span>
    </div>
    <el-table
      v-loading="loading"
      :data="rows"
      class="table-panel__table"
      empty-text="暂无内核"
    >
      <el-table-column prop="coreName" label="内核名称" min-width="190" show-overflow-tooltip />
      <el-table-column label="版本" width="140">
        <template #default="{ row }">
          {{ row.chromeVersion || '-' }}
        </template>
      </el-table-column>
      <el-table-column prop="corePath" label="路径" min-width="300" show-overflow-tooltip />
      <el-table-column prop="instanceCount" label="实例数" width="90" align="center" />
      <el-table-column label="状态" width="110">
        <template #default="{ row }">
          <el-tooltip :content="row.pathMessage || '路径可用'" placement="top">
            <el-tag :type="row.pathValid ? 'success' : 'danger'" effect="plain">
              {{ row.pathValid ? '可用' : '异常' }}
            </el-tag>
          </el-tooltip>
        </template>
      </el-table-column>
      <el-table-column label="默认" width="100">
        <template #default="{ row }">
          <el-tag v-if="row.isDefault" type="success" effect="plain">默认</el-tag>
          <el-button v-else size="small" link type="primary" @click="emit('setDefault', row)">
            设为默认
          </el-button>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="152" fixed="right">
        <template #default="{ row }">
          <el-button size="small" :icon="Edit" plain @click="emit('edit', row)">编辑</el-button>
          <el-button size="small" type="danger" plain @click="emit('delete', row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>
