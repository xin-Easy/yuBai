import type { BrowserFingerprint } from '@/types'

export type FingerprintGroupKey = 'basic' | 'screen' | 'hardware' | 'privacy'
export type FingerprintKey = keyof BrowserFingerprint

export interface FingerprintGroupConfig {
  key: FingerprintGroupKey
  title: string
  description: string
  fields: FingerprintKey[]
}

export interface SelectOption<T = string | number> {
  label: string
  value: T
}

export function defaultFingerprint(): BrowserFingerprint {
  return {
    browserBrand: 'Chrome',
    os: 'Windows',
    country: '',
    language: 'zh-CN',
    userAgent: '',
    acceptLanguage: 'zh-CN,zh;q=0.9,en;q=0.8',
    platform: 'Win32',
    vendor: 'Google Inc.',
    timezone: 'Asia/Shanghai',
    resolutionWidth: 1920,
    resolutionHeight: 1080,
    availWidth: 1920,
    availHeight: 1040,
    windowWidth: 1280,
    windowHeight: 900,
    windowX: 80,
    windowY: 60,
    deviceScaleFactor: 1,
    colorDepth: 24,
    cpuCores: 8,
    gpuMemory: 8,
    diskType: 'SSD',
    keyboardLayout: '',
    webglRenderer: '',
    webglVendor: '',
    canvasPolicy: 'real',
    webglPolicy: 'custom',
    webRtcPolicy: 'proxy',
    geoLatitude: 0,
    geoLongitude: 0,
    geoAccuracy: 100,
    geoPolicy: 'ask',
    fontPolicy: 'system',
    fontList: ['Arial', 'Calibri', 'Times New Roman', 'Microsoft YaHei', 'SimSun'],
    microphoneCount: 1,
    speakerCount: 1,
    cameraCount: 0,
    startMaximized: false,
    audioCodec: true,
    camera: false,
    doNotTrack: false,
    uaRandom: false,
    ldpEnabled: false,
  }
}

export const fingerprintGroups: FingerprintGroupConfig[] = [
  {
    key: 'basic',
    title: '基础环境',
    description: 'UA、语言、时区和平台画像',
    fields: [
      'browserBrand',
      'os',
      'country',
      'language',
      'acceptLanguage',
      'platform',
      'vendor',
      'timezone',
      'userAgent',
    ],
  },
  {
    key: 'screen',
    title: '屏幕与窗口',
    description: '分辨率、窗口大小和像素比',
    fields: [
      'resolutionWidth',
      'resolutionHeight',
      'availWidth',
      'availHeight',
      'windowWidth',
      'windowHeight',
      'windowX',
      'windowY',
      'deviceScaleFactor',
      'colorDepth',
      'startMaximized',
    ],
  },
  {
    key: 'hardware',
    title: '硬件与图形',
    description: 'CPU、GPU、WebGL 和 Canvas',
    fields: [
      'cpuCores',
      'gpuMemory',
      'diskType',
      'keyboardLayout',
      'webglRenderer',
      'webglVendor',
      'canvasPolicy',
      'webglPolicy',
    ],
  },
  {
    key: 'privacy',
    title: '隐私与设备 API',
    description: 'WebRTC、Geo、字体和媒体设备',
    fields: [
      'webRtcPolicy',
      'geoPolicy',
      'geoLatitude',
      'geoLongitude',
      'geoAccuracy',
      'fontPolicy',
      'fontList',
      'microphoneCount',
      'speakerCount',
      'cameraCount',
      'audioCodec',
      'camera',
      'doNotTrack',
      'uaRandom',
      'ldpEnabled',
    ],
  },
]

export const fingerprintGroupMap = Object.fromEntries(
  fingerprintGroups.map((group) => [group.key, group]),
) as Record<FingerprintGroupKey, FingerprintGroupConfig>

export const browserBrandOptions: SelectOption[] = [
  { label: 'Chrome', value: 'Chrome' },
  { label: 'Edge', value: 'Edge' },
  { label: 'Firefox', value: 'Firefox' },
  { label: 'Safari', value: 'Safari' },
  { label: 'Opera', value: 'Opera' },
]

export const osOptions: SelectOption[] = [
  { label: 'Windows', value: 'Windows' },
  { label: 'macOS', value: 'macOS' },
  { label: 'Linux', value: 'Linux' },
  { label: 'Android', value: 'Android' },
  { label: 'iOS', value: 'iOS' },
]

export const languageOptions: SelectOption[] = [
  { label: '中文简体 (zh-CN)', value: 'zh-CN' },
  { label: '中文繁体 (zh-TW)', value: 'zh-TW' },
  { label: 'English (en-US)', value: 'en-US' },
  { label: '日本語 (ja-JP)', value: 'ja-JP' },
  { label: '한국어 (ko-KR)', value: 'ko-KR' },
  { label: 'Français (fr-FR)', value: 'fr-FR' },
  { label: 'Deutsch (de-DE)', value: 'de-DE' },
  { label: 'Español (es-ES)', value: 'es-ES' },
]

export const platformOptions: SelectOption[] = [
  { label: 'Win32', value: 'Win32' },
  { label: 'Win64', value: 'Win64' },
  { label: 'MacIntel', value: 'MacIntel' },
  { label: 'Linux x86_64', value: 'Linux x86_64' },
  { label: 'iPhone', value: 'iPhone' },
  { label: 'Android', value: 'Android' },
]

export const vendorOptions: SelectOption[] = [
  { label: 'Google Inc.', value: 'Google Inc.' },
  { label: 'Apple Computer, Inc.', value: 'Apple Computer, Inc.' },
  { label: '空值', value: '' },
]

export const timezoneOptions: SelectOption[] = [
  { label: 'Asia/Shanghai (UTC+8)', value: 'Asia/Shanghai' },
  { label: 'America/New_York (UTC-5)', value: 'America/New_York' },
  { label: 'America/Los_Angeles (UTC-8)', value: 'America/Los_Angeles' },
  { label: 'Europe/London (UTC+0)', value: 'Europe/London' },
  { label: 'Europe/Berlin (UTC+1)', value: 'Europe/Berlin' },
  { label: 'Asia/Tokyo (UTC+9)', value: 'Asia/Tokyo' },
  { label: 'Australia/Sydney (UTC+10)', value: 'Australia/Sydney' },
  { label: 'Asia/Singapore (UTC+8)', value: 'Asia/Singapore' },
]

export const scaleFactorOptions: SelectOption<number>[] = [
  { label: '1.0', value: 1 },
  { label: '1.25', value: 1.25 },
  { label: '1.5', value: 1.5 },
  { label: '2.0', value: 2 },
  { label: '3.0', value: 3 },
]

export const colorDepthOptions: SelectOption<number>[] = [
  { label: '24 位 (标准)', value: 24 },
  { label: '16 位', value: 16 },
  { label: '32 位', value: 32 },
]

export const gpuMemoryOptions: SelectOption<number>[] = [4, 8, 12, 16, 24, 32].map((value) => ({
  label: `${value} GB`,
  value,
}))

export const diskTypeOptions: SelectOption[] = [
  { label: 'SSD', value: 'SSD' },
  { label: 'HDD', value: 'HDD' },
  { label: 'NVMe SSD', value: 'NVMe' },
]

export const canvasPolicyOptions: SelectOption[] = [
  { label: '真实', value: 'real' },
  { label: '噪声', value: 'noise' },
  { label: '阻断', value: 'block' },
]

export const webglPolicyOptions: SelectOption[] = [
  { label: '真实', value: 'real' },
  { label: '自定义', value: 'custom' },
  { label: '阻断', value: 'block' },
]

export const webRtcPolicyOptions: SelectOption[] = [
  { label: '仅代理', value: 'proxy' },
  { label: '禁用', value: 'disable' },
  { label: '真实', value: 'real' },
]

export const geoPolicyOptions: SelectOption[] = [
  { label: '询问', value: 'ask' },
  { label: '自定义', value: 'custom' },
  { label: '禁用', value: 'disable' },
]

export const fontPolicyOptions: SelectOption[] = [
  { label: '系统默认', value: 'system' },
  { label: '自定义列表', value: 'custom' },
  { label: '最小字体集', value: 'minimal' },
]

export const fontOptions: SelectOption[] = [
  'Arial',
  'Calibri',
  'Times New Roman',
  'Microsoft YaHei',
  'SimSun',
  'PingFang SC',
  'Helvetica Neue',
].map((value) => ({ label: value, value }))
