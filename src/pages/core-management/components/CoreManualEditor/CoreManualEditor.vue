<script setup lang="ts">
import { Search } from '@element-plus/icons-vue'
import type { BrowserCoreInput } from '@/types'
import type { PathValidationState } from '@/types/core-management'

defineProps<{
  form: BrowserCoreInput
  pathValidation: PathValidationState
}>()

const emit = defineEmits<{
  validate: []
}>()
</script>

<template>
  <div class="editor-section">
    <div class="editor-manual-layout">
      <div class="editor-panel">
        <el-form label-width="96px">
          <el-form-item label="内核名称" required>
            <el-input v-model="form.coreName" placeholder="例如：Chrome 136 Stable" />
          </el-form-item>
          <el-form-item label="内核路径" required>
            <div class="field-row">
              <el-input
                v-model="form.corePath"
                placeholder="填写 chrome.exe，或包含 chrome.exe 的目录"
              />
              <el-button :icon="Search" @click="emit('validate')">校验</el-button>
            </div>
          </el-form-item>
          <el-alert
            v-if="pathValidation"
            class="editor-alert"
            :type="pathValidation.valid ? 'success' : 'warning'"
            :title="pathValidation.message"
            show-icon
            :closable="false"
          />
          <el-form-item label="设为默认">
            <el-switch v-model="form.isDefault" />
          </el-form-item>
        </el-form>
      </div>

      <div class="page-stack">
        <div class="soft-card side-card">
          <div class="side-card__label">适合场景</div>
          <div class="side-card__text">
            适合已经有固定 Chrome 或 Chromium 目录的环境，录入快，也方便后续维护。
          </div>
        </div>
        <div class="soft-card side-card">
          <div class="side-card__label">路径说明</div>
          <div class="side-card__text">
            可以直接填写 <code>chrome.exe</code>，也可以填写安装目录，系统会自动补齐可执行文件。
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.editor-manual-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 280px;
  gap: 18px;
}

.editor-panel {
  @include glass-panel(rgba(255, 255, 255, 0.68));
}

.editor-panel :deep(.el-form) {
  max-width: 760px;
}

.editor-panel code {
  color: var(--primary-strong);
  font-size: 12px;
}

.field-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  width: 100%;
}

.editor-alert {
  margin: 0 0 16px 96px;
}

@include two-column-stack {
  .editor-manual-layout {
    grid-template-columns: 1fr;
  }

  .field-row {
    grid-template-columns: 1fr;
  }

  .editor-alert {
    margin-left: 0;
  }
}
</style>
