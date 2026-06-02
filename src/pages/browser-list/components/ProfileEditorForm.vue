<script setup lang="ts">
import { computed, ref } from 'vue'
import { ArrowDown, Close, WarningFilled } from '@element-plus/icons-vue'
import type { BrowserCore, BrowserFingerprint, BrowserProfileInput, BrowserProxy } from '@/types'
import type { ProxyGeo } from '../fingerprint-randomizer'
import FingerprintEditor from './FingerprintEditor.vue'

type ProfileEditorFormModel = BrowserProfileInput & {
  fingerprint: BrowserFingerprint
}

const SYSTEM_DIRECT_PROXY_ID = 'proxy-direct'

const props = defineProps<{
  form: ProfileEditorFormModel
  cores: BrowserCore[]
  proxies: BrowserProxy[]
  mode: 'create' | 'edit'
}>()

const proxyPopoverVisible = ref(false)

const standardProxies = computed(() => props.proxies.filter((item) => !item.sourceId))
const subscriptionProxies = computed(() => props.proxies.filter((item) => !!item.sourceId))

const selectedProxyName = computed(() => {
  if (!props.form.proxyId) return ''
  return props.proxies.find((item) => item.proxyId === props.form.proxyId)?.proxyName ?? ''
})

const selectedProxy = computed(() => {
  if (!props.form.proxyId) return null
  return props.proxies.find((item) => item.proxyId === props.form.proxyId) ?? null
})

const selectedProxyGeo = computed(() => parseProxyGeo(selectedProxy.value?.lastIPHealthJson))

const coreHelpText = computed(() => {
  if (props.mode === 'create') {
    return `当前可用内核 ${props.cores.length} 个。没有内核时，先去“内核管理”添加本机 Chrome 或在线下载。`
  }
  return `当前可用内核 ${props.cores.length} 个。修改后下次启动实例时生效。`
})

const baseInfoCopy = computed(() =>
  props.mode === 'create'
    ? '填写实例名称并确认启动内核，用户目录会按后端规则自动生成。'
    : '维护实例名称、用户目录和启动内核。',
)

const proxyHelpText = computed(() => {
  if (props.mode === 'create') {
    return `当前可用代理 ${props.proxies.length} 个。绑定后会把代理配置写入实例启动代理策略。`
  }
  return `当前可用代理 ${props.proxies.length} 个。修改代理后下次启动实例时生效。`
})

const launchHelpText = computed(() =>
  props.mode === 'create'
    ? '默认保留 --disable-sync 和 --no-first-run，减少首次启动干扰。'
    : '修改启动参数后，下次启动该实例时生效。',
)

function selectProxy(proxy: BrowserProxy) {
  if (props.form.proxyId === proxy.proxyId) {
    clearProxy()
    return
  }
  props.form.proxyId = proxy.proxyId
  props.form.proxyConfig = proxy.proxyConfig
  proxyPopoverVisible.value = false
}

function clearProxy() {
  props.form.proxyId = ''
  props.form.proxyConfig = ''
  proxyPopoverVisible.value = false
}

function isSystemDirectProxy(proxy: BrowserProxy) {
  return proxy.proxyId === SYSTEM_DIRECT_PROXY_ID
}

function parseProxyGeo(value?: string): ProxyGeo | null {
  if (!value?.trim()) {
    return null
  }
  try {
    const parsed = JSON.parse(value) as {
      lat?: number
      lon?: number
      country?: string
      regionName?: string
      city?: string
      timezone?: string
    }
    if (typeof parsed.lat !== 'number' || typeof parsed.lon !== 'number') {
      return null
    }
    return {
      lat: parsed.lat,
      lon: parsed.lon,
      accuracy: 100,
      country: parsed.country ?? '',
      regionName: parsed.regionName ?? '',
      city: parsed.city ?? '',
      timezone: parsed.timezone ?? '',
    }
  } catch {
    return null
  }
}

</script>

<template>
  <el-form class="profile-editor-form" label-width="108px">
    <section class="profile-form-section">
      <div class="profile-form-section__head">
        <div>
          <h3 class="profile-form-section__title">基础信息</h3>
          <p class="profile-form-section__copy">{{ baseInfoCopy }}</p>
        </div>
      </div>

      <el-form-item label="实例名称" required>
        <el-input v-model="form.profileName" placeholder="例如：店铺 A / 账号 01" />
      </el-form-item>
      <el-form-item v-if="mode === 'edit'" label="用户目录">
        <el-input v-model="form.userDataDir" placeholder="留空后由后端按 profileId 自动分配" />
      </el-form-item>
      <el-form-item>
        <template #label>
          <span class="form-label-with-help">
            浏览器内核
            <el-tooltip :content="coreHelpText" placement="top">
              <el-icon class="field-help-icon"><WarningFilled /></el-icon>
            </el-tooltip>
          </span>
        </template>
        <el-select v-model="form.coreId" class="w-full" clearable filterable placeholder="选择浏览器内核">
          <el-option
            v-for="core in cores"
            :key="core.coreId"
            :label="core.coreName"
            :value="core.coreId"
          >
            <div class="core-option">
              <span class="core-option__name">{{ core.coreName }}</span>
              <el-tag v-if="core.isDefault" class="core-option__tag" size="small" effect="plain" type="success">
                默认
              </el-tag>
            </div>
          </el-option>
        </el-select>
      </el-form-item>
    </section>

    <section class="profile-form-section">
      <div class="profile-form-section__head">
        <div>
          <h3 class="profile-form-section__title">网络出口</h3>
          <p class="profile-form-section__copy">选择代理后会自动填充配置，也可以手动补充。</p>
        </div>
      </div>

      <el-form-item>
        <template #label>
          <span class="form-label-with-help">
            代理
            <el-tooltip :content="proxyHelpText" placement="top">
              <el-icon class="field-help-icon"><WarningFilled /></el-icon>
            </el-tooltip>
          </span>
        </template>
        <el-popover
          :visible="proxyPopoverVisible"
          placement="bottom-start"
          :width="520"
          :show-arrow="false"
          :offset="6"
          popper-class="proxy-picker-popper"
          :popper-style="{ padding: 0 }"
        >
          <template #reference>
            <div class="proxy-trigger" @click="proxyPopoverVisible = !proxyPopoverVisible">
              <span v-if="form.proxyId" class="proxy-trigger__label">{{ selectedProxyName }}</span>
              <span v-else class="proxy-trigger__placeholder">选择代理节点</span>
              <el-icon v-if="form.proxyId" class="proxy-trigger__clear" @click.stop="clearProxy"><Close /></el-icon>
              <el-icon v-else class="proxy-trigger__arrow"><ArrowDown /></el-icon>
            </div>
          </template>
          <div class="proxy-picker">
            <div class="proxy-picker__col">
              <div class="proxy-picker__header">标准代理</div>
              <div class="proxy-picker__list">
                <div
                  v-for="proxy in standardProxies"
                  :key="proxy.proxyId"
                  class="proxy-picker__item"
                  :class="{ 'proxy-picker__item--active': form.proxyId === proxy.proxyId }"
                  @click="selectProxy(proxy)"
                >
                  <span class="proxy-picker__name">
                    {{ proxy.proxyName }}
                    <el-tag v-if="isSystemDirectProxy(proxy)" size="small" effect="plain" type="success">默认</el-tag>
                  </span>
                  <span class="proxy-picker__config">
                    {{ isSystemDirectProxy(proxy) ? 'direct / 不走代理' : proxy.proxyConfig }}
                  </span>
                </div>
                <div v-if="standardProxies.length === 0" class="proxy-picker__empty">暂无标准代理</div>
              </div>
            </div>
            <div class="proxy-picker__col">
              <div class="proxy-picker__header">订阅代理</div>
              <div class="proxy-picker__list">
                <div
                  v-for="proxy in subscriptionProxies"
                  :key="proxy.proxyId"
                  class="proxy-picker__item"
                  :class="{ 'proxy-picker__item--active': form.proxyId === proxy.proxyId }"
                  @click="selectProxy(proxy)"
                >
                  <span class="proxy-picker__name">{{ proxy.proxyName }}</span>
                  <span class="proxy-picker__config">{{ proxy.proxyConfig }}</span>
                </div>
                <div v-if="subscriptionProxies.length === 0" class="proxy-picker__empty">暂无订阅代理</div>
              </div>
            </div>
          </div>
        </el-popover>
      </el-form-item>
      <el-form-item label="代理配置">
        <el-input
          v-model="form.proxyConfig"
          type="textarea"
          :rows="2"
          placeholder="选择代理后会自动带入，也可以手动补充"
        />
      </el-form-item>
    </section>

    <FingerprintEditor
      v-model:fingerprint="form.fingerprint"
      :proxy-id="form.proxyId"
      :proxy-geo="selectedProxyGeo"
    />

    <section class="profile-form-section">
      <div class="profile-form-section__head">
        <div>
          <h3 class="profile-form-section__title">启动附加项</h3>
          <p class="profile-form-section__copy">补充启动参数和标签，用于批量管理与筛选。</p>
        </div>
      </div>

      <el-form-item>
        <template #label>
          <span class="form-label-with-help">
            启动参数
            <el-tooltip :content="launchHelpText" placement="top">
              <el-icon class="field-help-icon"><WarningFilled /></el-icon>
            </el-tooltip>
          </span>
        </template>
        <el-select
          v-model="form.launchArgs"
          class="w-full"
          multiple
          filterable
          allow-create
          default-first-option
          placeholder="输入参数后回车"
        >
          <el-option label="--disable-sync" value="--disable-sync" />
          <el-option label="--no-first-run" value="--no-first-run" />
        </el-select>
      </el-form-item>
      <el-form-item label="标签">
        <el-select
          v-model="form.tags"
          class="w-full"
          multiple
          filterable
          allow-create
          default-first-option
          placeholder="输入标签后回车"
        />
      </el-form-item>
    </section>
  </el-form>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.profile-editor-form {
  display: grid;
  gap: 16px;
}

.profile-form-section {
  padding: 18px;
  @include glass-panel(rgba(255, 255, 255, 0.68));
}

.profile-form-section__head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.profile-form-section__title {
  margin: 0;
  color: var(--text);
  font-size: 15px;
  font-weight: 750;
}

.profile-form-section__copy {
  margin: 5px 0 0;
  color: var(--text-soft);
  font-size: 12px;
  line-height: 1.6;
}

.form-label-with-help {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.field-help-icon {
  color: var(--el-color-warning);
  cursor: help;
  font-size: 14px;
  transform: translateY(1px);
}
.core-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  min-width: 0;
}
.core-option__name {
  min-width: 0;
  overflow: hidden;
  color: var(--text);
  text-overflow: ellipsis;
  white-space: nowrap;
}
.core-option__tag {
  flex: 0 0 auto;
}
.proxy-trigger {
  width: 100%;
  min-height: 36px;
  padding: 0 30px 0 12px;
  border: 1px solid rgba(102, 128, 139, 0.2);
  border-radius: var(--el-border-radius-base);
  background: rgba(255, 255, 255, 0.82);
  cursor: pointer;
  position: relative;
  display: flex;
  align-items: center;
  transition: border-color 0.2s;
}
.proxy-trigger:hover {
  border-color: rgba(15, 159, 122, 0.36);
}
.proxy-trigger__label {
  font-size: 14px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.proxy-trigger__placeholder {
  font-size: 14px;
  color: var(--el-text-color-placeholder);
}
.proxy-trigger__arrow,
.proxy-trigger__clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 14px;
  color: var(--el-text-color-placeholder);
}
.proxy-trigger__clear:hover {
  color: var(--text-soft);
}
.proxy-picker {
  display: flex;
  gap: 0;
  overflow: hidden;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.96);
}
.proxy-picker__col {
  flex: 1;
  min-width: 0;
}
.proxy-picker__col + .proxy-picker__col {
  border-left: 1px solid var(--border);
}
.proxy-picker__header {
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 750;
  color: var(--text);
  background: rgba(247, 252, 250, 0.9);
  border-bottom: 1px solid var(--border);
}
.proxy-picker__list {
  max-height: 240px;
  overflow-y: auto;
}
.proxy-picker__item {
  padding: 10px 12px;
  cursor: pointer;
  border-bottom: 1px solid rgba(131, 151, 160, 0.12);
  transition: background 0.15s, border-color 0.15s;
}
.proxy-picker__item:hover {
  background: rgba(238, 251, 245, 0.72);
}
.proxy-picker__item--active {
  background: var(--primary-soft);
  border-left: 3px solid var(--primary);
}
.proxy-picker__name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.proxy-picker__config {
  display: block;
  font-size: 11px;
  color: var(--text-soft);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}
.proxy-picker__empty {
  padding: 24px 12px;
  text-align: center;
  font-size: 13px;
  color: var(--el-text-color-placeholder);
}

@include respond-down(980px) {
  .proxy-picker {
    flex-direction: column;
  }

  .proxy-picker__col + .proxy-picker__col {
    border-left: 0;
    border-top: 1px solid var(--border);
  }
}

@include mobile {
  .profile-form-section {
    padding: 16px;
  }

  .profile-form-section :deep(.el-form-item__label) {
    width: 100% !important;
    justify-content: flex-start;
    margin-bottom: 6px;
  }

  .profile-form-section :deep(.el-form-item__content) {
    margin-left: 0 !important;
  }
}
</style>
