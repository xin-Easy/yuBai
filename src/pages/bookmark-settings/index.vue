<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Upload } from '@element-plus/icons-vue'
import {
  fetchBookmarks,
  resetBookmarks,
  saveBookmarks,
  syncBookmarksToProfiles,
} from '@/api/bookmarks'
import { usePageRefresh } from '@/composables/usePageRefresh'
import type { BrowserBookmark } from '@/types'

const loading = ref(false)
const saving = ref(false)
const dialogOpen = ref(false)
const bookmarks = ref<BrowserBookmark[]>([])

const form = reactive<BrowserBookmark>({
  name: '',
  url: '',
  openOnStart: false,
})

const startupCount = computed(() => bookmarks.value.filter((item) => item.openOnStart).length)

async function loadBookmarks() {
  loading.value = true
  try {
    bookmarks.value = await fetchBookmarks()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '加载书签失败')
  } finally {
    loading.value = false
  }
}

function openCreateDialog() {
  form.name = ''
  form.url = ''
  form.openOnStart = false
  dialogOpen.value = true
}

async function persist(next: BrowserBookmark[]) {
  saving.value = true
  try {
    await saveBookmarks(next)
    bookmarks.value = next
    ElMessage.success('书签已保存')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '保存书签失败')
  } finally {
    saving.value = false
  }
}

async function handleAdd() {
  if (!form.name.trim() || !form.url.trim()) {
    ElMessage.warning('请填写书签名称和 URL')
    return
  }
  await persist([...bookmarks.value, { ...form, name: form.name.trim(), url: form.url.trim() }])
  dialogOpen.value = false
}

async function handleDelete(index: number) {
  const next = bookmarks.value.filter((_, itemIndex) => itemIndex !== index)
  await persist(next)
}

async function handleReset() {
  await ElMessageBox.confirm('确认清空所有默认书签吗？', '重置确认', {
    type: 'warning',
    confirmButtonText: '清空',
    cancelButtonText: '取消',
  })
  await resetBookmarks()
  bookmarks.value = []
  ElMessage.success('默认书签已清空')
}

async function handleSync() {
  const result = await syncBookmarksToProfiles()
  ElMessage.success(`同步任务已触发：总数 ${result.total}，已同步 ${result.synced}`)
}

onMounted(loadBookmarks)
usePageRefresh(loadBookmarks)
</script>

<template>
  <section class="split-layout">
    <div class="page-header">
      <div>
        <h1 class="page-title">默认书签</h1>
        <p class="page-subtitle">维护新实例使用的默认书签集合，需要时再同步到已有实例。</p>
      </div>
      <div class="page-actions">
        <el-button :icon="Upload" :loading="saving" @click="handleSync">同步到实例</el-button>
        <el-button type="primary" :icon="Plus" @click="openCreateDialog">添加书签</el-button>
      </div>
    </div>

    <div class="page-grid-3">
      <div class="metric-card">
        <div class="metric-card__label">书签总数</div>
        <div class="metric-card__value">{{ bookmarks.length }}</div>
        <div class="metric-card__text">默认注入集合</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">启动时打开</div>
        <div class="metric-card__value">{{ startupCount }}</div>
        <div class="metric-card__text">随新实例自动打开</div>
      </div>
      <div class="metric-card">
        <div class="metric-card__label">同步方式</div>
        <div class="metric-card__text">按需手动同步到现有实例</div>
      </div>
    </div>

    <div class="content-panel table-panel">
      <div class="table-panel__meta">
        <span class="muted-text">{{ bookmarks.length }} 条书签</span>
        <span class="muted-text">名称 / 地址 / 启动策略</span>
      </div>
      <el-table v-loading="loading" :data="bookmarks" class="table-panel__table" empty-text="暂无默认书签">
        <el-table-column prop="name" label="名称" min-width="220" show-overflow-tooltip />
        <el-table-column prop="url" label="URL" min-width="320" show-overflow-tooltip />
        <el-table-column label="启动时打开" width="120">
          <template #default="{ row }">
            <el-tag :type="row.openOnStart ? 'success' : 'info'" effect="plain">
              {{ row.openOnStart ? '是' : '否' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ $index }">
            <el-button size="small" type="danger" plain @click="handleDelete($index)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <div class="page-actions">
      <el-button type="danger" plain @click="handleReset">清空默认书签</el-button>
    </div>

    <el-dialog v-model="dialogOpen" title="添加书签" width="min(560px, 92vw)" destroy-on-close>
      <el-form label-width="96px">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" placeholder="例如：IP 检测" />
        </el-form-item>
        <el-form-item label="URL" required>
          <el-input v-model="form.url" placeholder="https://example.com" />
        </el-form-item>
        <el-form-item label="启动时打开">
          <el-switch v-model="form.openOnStart" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogOpen = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleAdd">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<style scoped>
:deep(.el-dialog__body) {
  padding-top: 18px;
}
</style>
