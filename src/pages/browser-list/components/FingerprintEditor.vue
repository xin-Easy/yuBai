<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessage } from 'element-plus'
import type { BrowserFingerprint } from '@/types'
import { randomizeBrowserFingerprint, type ProxyGeo } from '../fingerprint-randomizer'
import {
  browserBrandOptions,
  canvasPolicyOptions,
  colorDepthOptions,
  defaultFingerprint,
  diskTypeOptions,
  fingerprintGroupMap,
  fontOptions,
  fontPolicyOptions,
  geoPolicyOptions,
  gpuMemoryOptions,
  languageOptions,
  osOptions,
  platformOptions,
  scaleFactorOptions,
  timezoneOptions,
  vendorOptions,
  webglPolicyOptions,
  webRtcPolicyOptions,
  type FingerprintGroupKey,
  type FingerprintKey,
} from '../fingerprint-config'
import FingerprintPanelTitle from './FingerprintPanelTitle.vue'

const props = defineProps<{
  proxyId: string
  proxyGeo: ProxyGeo | null
}>()

const fingerprint = defineModel<BrowserFingerprint>('fingerprint', { required: true })
const activeFingerprintPanels = ref(['basic', 'screen'])

const selectedProxyGeo = computed(() => props.proxyGeo)

function patchFingerprint(fields: FingerprintKey[], source: BrowserFingerprint) {
  const next = { ...fingerprint.value }
  for (const field of fields) {
    Object.assign(next, { [field]: source[field] })
  }
  fingerprint.value = next
}

function applyProxyGeo() {
  const geo = selectedProxyGeo.value
  if (!geo) {
    ElMessage.warning('当前绑定代理没有可用的出口地理位置，请先在代理池执行 IP 健康检测')
    return
  }

  fingerprint.value = {
    ...fingerprint.value,
    geoLatitude: geo.lat,
    geoLongitude: geo.lon,
    geoAccuracy: geo.accuracy,
    geoPolicy: 'custom',
    ...(geo.timezone ? { timezone: geo.timezone } : {}),
    ...(geo.country ? { country: [geo.country, geo.regionName, geo.city].filter(Boolean).join(' / ') } : {}),
  }
  ElMessage.success('已从绑定代理填充 GeoLocation')
}

function resetGroup(group: FingerprintGroupKey) {
  patchFingerprint(fingerprintGroupMap[group].fields, defaultFingerprint())
  ElMessage.success('已重置当前分组')
}

function randomizeGroup(group: FingerprintGroupKey) {
  const randomized = randomizeBrowserFingerprint(fingerprint.value, selectedProxyGeo.value)
  patchFingerprint(fingerprintGroupMap[group].fields, randomized)
  ElMessage.success('已随机生成当前分组')
}
</script>

<template>
  <div class="fingerprint-section">
    <div class="fingerprint-section__header">
      <span>指纹配置</span>
      <span class="fingerprint-section__hint">每个分组可独立编辑、随机和重置</span>
    </div>
    <el-collapse v-model="activeFingerprintPanels" class="fingerprint-collapse">
      <el-collapse-item name="basic">
        <template #title>
          <FingerprintPanelTitle
            :group="fingerprintGroupMap.basic"
            @reset="resetGroup('basic')"
            @randomize="randomizeGroup('basic')"
          />
        </template>
        <div class="fingerprint-grid">
      <el-form-item label="浏览器品牌" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.browserBrand" placeholder="选择品牌">
          <el-option v-for="option in browserBrandOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="操作系统" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.os" placeholder="选择系统">
          <el-option v-for="option in osOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="国家/地区" class="fingerprint-grid__item">
        <el-input v-model="fingerprint.country" placeholder="例如：美国、日本" />
      </el-form-item>
      <el-form-item label="语言" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.language" filterable allow-create placeholder="选择语言">
          <el-option v-for="option in languageOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="Accept-Language" class="fingerprint-grid__item">
        <el-input v-model="fingerprint.acceptLanguage" placeholder="zh-CN,zh;q=0.9,en;q=0.8" />
      </el-form-item>
      <el-form-item label="JS Platform" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.platform" filterable allow-create placeholder="选择 Platform">
          <el-option v-for="option in platformOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="Vendor" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.vendor" filterable allow-create placeholder="选择 Vendor">
          <el-option v-for="option in vendorOptions" :key="option.label" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="时区" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.timezone" filterable allow-create placeholder="选择时区">
          <el-option v-for="option in timezoneOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="User-Agent" class="fingerprint-grid__item fingerprint-grid__item--wide">
        <el-input
          v-model="fingerprint.userAgent"
          type="textarea"
          :rows="2"
          placeholder="留空则使用浏览器默认 UA；填写后会写入 --user-agent"
        />
      </el-form-item>
        </div>
      </el-collapse-item>

      <el-collapse-item name="screen">
        <template #title>
          <FingerprintPanelTitle
            :group="fingerprintGroupMap.screen"
            @reset="resetGroup('screen')"
            @randomize="randomizeGroup('screen')"
          />
        </template>
        <div class="fingerprint-grid">
      <el-form-item label="分辨率" class="fingerprint-grid__item">
        <div class="resolution-inputs">
          <el-input-number v-model="fingerprint.resolutionWidth" :min="640" :max="7680" :step="1" controls-position="right" />
          <span class="resolution-sep">×</span>
          <el-input-number v-model="fingerprint.resolutionHeight" :min="480" :max="4320" :step="1" controls-position="right" />
        </div>
      </el-form-item>
      <el-form-item label="可用区域" class="fingerprint-grid__item">
        <div class="resolution-inputs">
          <el-input-number v-model="fingerprint.availWidth" :min="640" :max="7680" :step="1" controls-position="right" />
          <span class="resolution-sep">×</span>
          <el-input-number v-model="fingerprint.availHeight" :min="480" :max="4320" :step="1" controls-position="right" />
        </div>
      </el-form-item>
      <el-form-item label="窗口大小" class="fingerprint-grid__item">
        <div class="resolution-inputs">
          <el-input-number v-model="fingerprint.windowWidth" :min="640" :max="7680" :step="1" controls-position="right" />
          <span class="resolution-sep">×</span>
          <el-input-number v-model="fingerprint.windowHeight" :min="480" :max="4320" :step="1" controls-position="right" />
        </div>
      </el-form-item>
      <el-form-item label="窗口位置" class="fingerprint-grid__item">
        <div class="resolution-inputs">
          <el-input-number v-model="fingerprint.windowX" :min="0" :max="7680" :step="1" controls-position="right" />
          <span class="resolution-sep">,</span>
          <el-input-number v-model="fingerprint.windowY" :min="0" :max="4320" :step="1" controls-position="right" />
        </div>
      </el-form-item>
      <el-form-item label="像素比" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.deviceScaleFactor" placeholder="devicePixelRatio">
          <el-option v-for="option in scaleFactorOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="色深" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.colorDepth" placeholder="色深">
          <el-option v-for="option in colorDepthOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="最大化启动" class="fingerprint-grid__item">
        <el-switch v-model="fingerprint.startMaximized" active-text="启用" inactive-text="禁用" />
      </el-form-item>
        </div>
      </el-collapse-item>

      <el-collapse-item name="hardware">
        <template #title>
          <FingerprintPanelTitle
            :group="fingerprintGroupMap.hardware"
            @reset="resetGroup('hardware')"
            @randomize="randomizeGroup('hardware')"
          />
        </template>
        <div class="fingerprint-grid">
      <el-form-item label="CPU 核心数" class="fingerprint-grid__item">
        <el-input-number v-model="fingerprint.cpuCores" :min="1" :max="128" :step="1" controls-position="right" />
      </el-form-item>
      <el-form-item label="显存大小" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.gpuMemory" placeholder="显存">
          <el-option v-for="option in gpuMemoryOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="硬盘类型" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.diskType" placeholder="硬盘类型">
          <el-option v-for="option in diskTypeOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="键盘布局" class="fingerprint-grid__item">
        <el-input v-model="fingerprint.keyboardLayout" placeholder="留空表示无键盘" />
      </el-form-item>
      <el-form-item label="WebGL 渲染器" class="fingerprint-grid__item">
        <el-input v-model="fingerprint.webglRenderer" placeholder="例如：ANGLE (NVIDIA...)" />
      </el-form-item>
      <el-form-item label="WebGL 供应商" class="fingerprint-grid__item">
        <el-input v-model="fingerprint.webglVendor" placeholder="例如：Google Inc. (NVIDIA)" />
      </el-form-item>
      <el-form-item label="Canvas 策略" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.canvasPolicy" placeholder="Canvas 策略">
          <el-option v-for="option in canvasPolicyOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="WebGL 策略" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.webglPolicy" placeholder="WebGL 策略">
          <el-option v-for="option in webglPolicyOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
        </div>
      </el-collapse-item>

      <el-collapse-item name="privacy">
        <template #title>
          <FingerprintPanelTitle
            :group="fingerprintGroupMap.privacy"
            @reset="resetGroup('privacy')"
            @randomize="randomizeGroup('privacy')"
          />
        </template>
        <div class="fingerprint-grid">
      <el-form-item label="WebRTC" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.webRtcPolicy" placeholder="WebRTC 策略">
          <el-option v-for="option in webRtcPolicyOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="Geo 策略" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.geoPolicy" placeholder="GeoLocation 策略">
          <el-option v-for="option in geoPolicyOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="GeoLocation" class="fingerprint-grid__item fingerprint-grid__item--wide">
        <div class="geo-editor">
          <div class="geo-editor__inputs">
            <el-input-number v-model="fingerprint.geoLatitude" :min="-90" :max="90" :precision="6" controls-position="right" placeholder="纬度" />
            <el-input-number v-model="fingerprint.geoLongitude" :min="-180" :max="180" :precision="6" controls-position="right" placeholder="经度" />
            <el-input-number v-model="fingerprint.geoAccuracy" :min="1" :max="10000" :step="10" controls-position="right" placeholder="精度" />
          </div>
          <div class="geo-editor__actions">
            <el-button size="small" :disabled="!proxyId" @click="applyProxyGeo">从绑定代理填充</el-button>
            <span v-if="selectedProxyGeo" class="geo-editor__hint">
              代理位置：{{ selectedProxyGeo.country || '-' }} {{ selectedProxyGeo.regionName || '' }} {{ selectedProxyGeo.city || '' }}
            </span>
            <span v-else class="geo-editor__hint">需要代理池 IP 健康检测结果中的 lat/lon</span>
          </div>
        </div>
      </el-form-item>
      <el-form-item label="字体策略" class="fingerprint-grid__item">
        <el-select v-model="fingerprint.fontPolicy" placeholder="字体策略">
          <el-option v-for="option in fontPolicyOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="字体列表" class="fingerprint-grid__item">
        <el-select
          v-model="fingerprint.fontList"
          class="w-full"
          multiple
          filterable
          allow-create
          default-first-option
          placeholder="输入字体名后回车"
        >
          <el-option v-for="option in fontOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="麦克风数量" class="fingerprint-grid__item">
        <el-input-number v-model="fingerprint.microphoneCount" :min="0" :max="8" :step="1" controls-position="right" />
      </el-form-item>
      <el-form-item label="扬声器数量" class="fingerprint-grid__item">
        <el-input-number v-model="fingerprint.speakerCount" :min="0" :max="8" :step="1" controls-position="right" />
      </el-form-item>
      <el-form-item label="摄像头数量" class="fingerprint-grid__item">
        <el-input-number v-model="fingerprint.cameraCount" :min="0" :max="8" :step="1" controls-position="right" />
      </el-form-item>
      <el-form-item label="Audio 编解码" class="fingerprint-grid__item">
        <el-switch v-model="fingerprint.audioCodec" active-text="支持" inactive-text="禁用" />
      </el-form-item>
      <el-form-item label="Camera 摄像头" class="fingerprint-grid__item">
        <el-switch v-model="fingerprint.camera" active-text="有" inactive-text="无" />
      </el-form-item>
      <el-form-item label="Do Not Track" class="fingerprint-grid__item">
        <el-switch v-model="fingerprint.doNotTrack" active-text="启用" inactive-text="禁用" />
      </el-form-item>
      <el-form-item label="UA/MTC 随机化" class="fingerprint-grid__item">
        <el-switch v-model="fingerprint.uaRandom" active-text="随机" inactive-text="固定" />
      </el-form-item>
      <el-form-item label="LDP 隐私保护" class="fingerprint-grid__item">
        <el-switch v-model="fingerprint.ldpEnabled" active-text="启用" inactive-text="禁用" />
      </el-form-item>
        </div>
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/mixins" as *;

.fingerprint-section {
  padding: 18px;
  @include glass-panel(rgba(255, 255, 255, 0.68));
}
.fingerprint-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
  font-size: 15px;
  font-weight: 750;
  color: var(--text);
}
.fingerprint-section__hint {
  color: var(--text-soft);
  font-size: 12px;
  font-weight: 500;
}
.fingerprint-collapse {
  border-top: 1px solid var(--border);
  border-bottom: 0;
}
.fingerprint-collapse :deep(.el-collapse-item__header) {
  min-height: 50px;
  padding: 0 10px;
  border-bottom-color: rgba(131, 151, 160, 0.14);
  background: transparent;
}

.fingerprint-collapse :deep(.el-collapse-item__wrap) {
  border-bottom-color: rgba(131, 151, 160, 0.14);
  background: transparent;
}

.fingerprint-collapse :deep(.el-collapse-item__title) {
  flex: 1;
  min-width: 0;
}
.fingerprint-collapse :deep(.el-collapse-item__arrow) {
  margin-left: 10px;
}
.fingerprint-collapse :deep(.el-collapse-item__content) {
  padding: 14px 10px 6px;
}
.fingerprint-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 2px 16px;
}
.fingerprint-grid__item--wide {
  grid-column: 1 / -1;
}
.fingerprint-grid__item :deep(.el-form-item__label) {
  width: 100%;
  color: var(--text-soft);
  font-size: 13px;
}
.fingerprint-grid__item :deep(.el-input),
.fingerprint-grid__item :deep(.el-select) {
  width: 100%;
}
.resolution-inputs {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
}
.resolution-inputs .el-input-number {
  flex: 1;
}
.resolution-sep {
  color: var(--text-muted);
  font-size: 13px;
  user-select: none;
}
.geo-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
.geo-editor__inputs {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
.geo-editor__actions {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  padding: 10px 12px;
  border: 1px solid rgba(131, 151, 160, 0.16);
  border-radius: 8px;
  background: rgba(247, 252, 250, 0.82);
}
.geo-editor__hint {
  color: var(--text-soft);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
@include two-column-stack {
  .fingerprint-grid {
    grid-template-columns: 1fr;
  }
}

@include respond-down(900px) {
  .fingerprint-section__header {
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
  }

  .geo-editor__inputs {
    grid-template-columns: 1fr;
  }
  .geo-editor__actions {
    align-items: flex-start;
    flex-direction: column;
  }
  .geo-editor__hint {
    white-space: normal;
  }
}
</style>
