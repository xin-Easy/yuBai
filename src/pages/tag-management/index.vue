<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import { deleteBrowserGroup, fetchBrowserGroups, saveBrowserGroup } from '@/api/groups'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { BrowserGroupInput, BrowserGroupWithCount } from '@/types'

const loading = ref(false)
const saving = ref(false)
const dialogOpen = ref(false)
const groups = ref<BrowserGroupWithCount[]>([])

const form = reactive<BrowserGroupInput>({
  groupId: '',
  groupName: '',
  color: '',
  sortOrder: 0,
})

const totalProfiles = computed(() => groups.value.reduce((sum, item) => sum + item.profileCount, 0))

async function loadGroups() {
  loading.value = true
  try {
    groups.value = await fetchBrowserGroups()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载标签分组失败')
  } finally {
    loading.value = false
  }
}

function openCreateDialog() {
  form.groupId = ''
  form.groupName = ''
  form.color = ''
  form.sortOrder = 0
  dialogOpen.value = true
}

async function handleSave() {
  if (!form.groupName.trim()) {
    ElMessage.warning('请填写分组名称')
    return
  }

  saving.value = true
  try {
    const saved = await saveBrowserGroup({ ...form, groupName: form.groupName.trim() })
    const existed = groups.value.some((item) => item.groupId === saved.groupId)
    if (existed) {
      groups.value = groups.value.map((item) =>
        item.groupId === saved.groupId ? { ...saved, profileCount: item.profileCount, instanceCount: item.instanceCount } : item,
      )
    } else {
      groups.value = [{ ...saved, profileCount: 0, instanceCount: 0 }, ...groups.value]
    }
    dialogOpen.value = false
    ElMessage.success('分组已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存分组失败')
  } finally {
    saving.value = false
  }
}

async function handleEdit(row: BrowserGroupWithCount) {
  form.groupId = row.groupId
  form.groupName = row.groupName
  form.color = row.color || ''
  form.sortOrder = row.sortOrder || 0
  dialogOpen.value = true
}

async function handleDelete(row: BrowserGroupWithCount) {
  await ElMessageBox.confirm(`确认删除分组"${row.groupName}"吗？`, '删除确认', {
    type: 'warning',
    confirmButtonText: '删除',
    cancelButtonText: '取消',
  })

  try {
    await deleteBrowserGroup(row.groupId)
    groups.value = groups.value.filter((item) => item.groupId !== row.groupId)
    ElMessage.success('分组已删除')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '删除分组失败')
  }
}

onMounted(loadGroups)
usePageRefresh(loadGroups)
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">标签管理</h1>
        <p class="page-subtitle">管理实例分组并汇总当前已使用的标签，方便快速筛选和归类。</p>
      </div>
      <div class="page-actions">
        <el-button type="primary" :icon="Plus" @click="openCreateDialog">新建分组</el-button>
      </div>
    </div>

    <div class="page-grid-3">
      <div class="metric-card">
        <div class="metric-card__label">分组数量</div>
        <div class="metric-card__value">{{ groups.length }}</div>
        <div class="metric-card__text">实例归类维度</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">关联实例</div>
        <div class="metric-card__value">{{ totalProfiles }}</div>
        <div class="metric-card__text">已纳入分组管理</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">空分组</div>
        <div class="metric-card__value">{{ groups.filter((item) => item.profileCount === 0).length }}</div>
        <div class="metric-card__text">可清理或继续配置</div>
      </div>
    </div>

    <div class="content-panel table-panel">
      <div class="table-panel__meta">
        <span class="muted-text">{{ groups.length }} 个分组</span>
        <span class="muted-text">颜色 / 实例数 / 排序</span>
      </div>
      <el-table v-loading="loading" :data="groups" class="table-panel__table" empty-text="暂无分组">
        <el-table-column prop="groupName" label="分组名称" min-width="220" show-overflow-tooltip />
        <el-table-column label="颜色" width="100">
          <template #default="{ row }">
            <span
              v-if="row.color"
              class="inline-block h-5 w-5 rounded"
              :style="{ background: row.color }"
            />
            <span v-else class="muted-text">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="profileCount" label="实例数" width="100" align="center" />
        <el-table-column prop="sortOrder" label="排序" width="80" align="center" />
        <el-table-column label="操作" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" plain @click="handleEdit(row)">编辑</el-button>
            <el-button size="small" type="danger" plain @click="handleDelete(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <el-dialog v-model="dialogOpen" title="新建分组" width="min(480px, 92vw)" destroy-on-close>
      <el-form label-width="96px">
        <el-form-item label="分组名称" required>
          <el-input v-model="form.groupName" placeholder="例如：电商 / 社交 / 测试" />
        </el-form-item>
        <el-form-item label="颜色">
          <el-color-picker v-model="form.color" />
        </el-form-item>
        <el-form-item label="排序">
          <el-input-number v-model="form.sortOrder" :min="0" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogOpen = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<style scoped>
:deep(.el-color-picker__trigger) {
  border-color: var(--border);
  background: rgba(255, 255, 255, 0.72);
}
</style>
