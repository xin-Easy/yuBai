import type { BrowserFingerprint } from '@/types'

export interface ProxyGeo {
  lat: number
  lon: number
  accuracy: number
  country: string
  regionName: string
  city: string
  timezone: string
}

interface RandomLocale {
  country: string
  language: string
  acceptLanguage: string
  timezone: string
  keyboardLayout: string
  geo: {
    lat: number
    lon: number
  }
  fonts: string[]
}

interface RandomDevice {
  os: string
  platform: string
  vendor: string
  uaPlatform: string
  scaleFactors: number[]
  resolutions: Array<{
    width: number
    height: number
  }>
  webglVendor: string
  webglRenderers: string[]
  fonts: string[]
}

export function randomizeBrowserFingerprint(current: BrowserFingerprint, proxyGeo: ProxyGeo | null): BrowserFingerprint {
  const device = pick(randomDevices)
  const locale = pick(randomLocales)
  const chromeMajor = randomInt(126, 134)
  const chromeBuild = randomInt(6400, 6999)
  const chromePatch = randomInt(80, 180)
  const resolution = pick(device.resolutions)
  const cameraCount = pick([0, 0, 1])
  const windowSize = pick([
    { width: 1280, height: 900 },
    { width: 1366, height: 900 },
    { width: 1440, height: 900 },
    { width: 1536, height: 960 },
    { width: 1600, height: 1000 },
  ])

  return {
    ...current,
    browserBrand: 'Chrome',
    os: device.os,
    platform: device.platform,
    vendor: device.vendor,
    language: locale.language,
    acceptLanguage: locale.acceptLanguage,
    timezone: proxyGeo?.timezone || locale.timezone,
    country: proxyGeo
      ? [proxyGeo.country, proxyGeo.regionName, proxyGeo.city].filter(Boolean).join(' / ')
      : locale.country,
    userAgent: buildChromeUserAgent(device.uaPlatform, chromeMajor, chromeBuild, chromePatch),
    resolutionWidth: resolution.width,
    resolutionHeight: resolution.height,
    availWidth: resolution.width,
    availHeight: Math.max(480, resolution.height - pick([40, 48, 72])),
    windowWidth: Math.min(windowSize.width, resolution.width),
    windowHeight: Math.min(windowSize.height, resolution.height),
    windowX: randomInt(0, 120),
    windowY: randomInt(0, 90),
    deviceScaleFactor: pick(device.scaleFactors),
    colorDepth: pick([24, 24, 24, 32]),
    cpuCores: pick([4, 6, 8, 8, 12, 16]),
    gpuMemory: pick([4, 8, 8, 12, 16]),
    diskType: pick(['SSD', 'SSD', 'NVMe']),
    keyboardLayout: locale.keyboardLayout,
    webglVendor: device.webglVendor,
    webglRenderer: pick(device.webglRenderers),
    canvasPolicy: pick(['real', 'noise']),
    webglPolicy: 'custom',
    webRtcPolicy: pick(['proxy', 'proxy', 'disable']),
    geoPolicy: proxyGeo ? 'custom' : pick(['ask', 'disable']),
    geoLatitude: proxyGeo?.lat ?? randomGeo(locale.geo.lat),
    geoLongitude: proxyGeo?.lon ?? randomGeo(locale.geo.lon),
    geoAccuracy: randomInt(30, 150),
    fontPolicy: 'custom',
    fontList: dedupeStrings([...device.fonts, ...locale.fonts]),
    microphoneCount: pick([0, 1, 1, 2]),
    speakerCount: pick([1, 1, 2]),
    cameraCount,
    camera: cameraCount > 0,
    audioCodec: true,
    doNotTrack: pick([false, false, true]),
    uaRandom: false,
    ldpEnabled: pick([false, true]),
    startMaximized: false,
  }
}

function buildChromeUserAgent(platform: string, major: number, build: number, patch: number) {
  return `Mozilla/5.0 (${platform}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${major}.0.${build}.${patch} Safari/537.36`
}

function randomGeo(base: number) {
  return Number((base + (Math.random() - 0.5) * 0.18).toFixed(6))
}

function randomInt(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function pick<T>(items: T[]) {
  return items[randomInt(0, items.length - 1)]
}

function dedupeStrings(items: string[]) {
  return [...new Set(items)]
}

const randomLocales: RandomLocale[] = [
  {
    country: '中国 / 广东 / 深圳',
    language: 'zh-CN',
    acceptLanguage: 'zh-CN,zh;q=0.9,en;q=0.8',
    timezone: 'Asia/Shanghai',
    keyboardLayout: 'zh-CN',
    geo: { lat: 22.5431, lon: 114.0579 },
    fonts: ['Microsoft YaHei', 'SimSun'],
  },
  {
    country: '日本 / 东京都 / 东京',
    language: 'ja-JP',
    acceptLanguage: 'ja-JP,ja;q=0.9,en-US;q=0.8,en;q=0.7',
    timezone: 'Asia/Tokyo',
    keyboardLayout: 'ja-JP',
    geo: { lat: 35.6895, lon: 139.6917 },
    fonts: ['Yu Gothic', 'Meiryo'],
  },
  {
    country: '美国 / California / Los Angeles',
    language: 'en-US',
    acceptLanguage: 'en-US,en;q=0.9',
    timezone: 'America/Los_Angeles',
    keyboardLayout: 'en-US',
    geo: { lat: 34.0522, lon: -118.2437 },
    fonts: ['Arial', 'Calibri', 'Times New Roman'],
  },
]

const randomDevices: RandomDevice[] = [
  {
    os: 'Windows',
    platform: 'Win32',
    vendor: 'Google Inc.',
    uaPlatform: 'Windows NT 10.0; Win64; x64',
    scaleFactors: [1, 1.25, 1.5],
    resolutions: [
      { width: 1920, height: 1080 },
      { width: 2560, height: 1440 },
      { width: 1536, height: 864 },
    ],
    webglVendor: 'Google Inc. (NVIDIA)',
    webglRenderers: [
      'ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0)',
      'ANGLE (Intel, Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0)',
    ],
    fonts: ['Arial', 'Calibri', 'Segoe UI', 'Times New Roman'],
  },
  {
    os: 'macOS',
    platform: 'MacIntel',
    vendor: 'Google Inc.',
    uaPlatform: 'Macintosh; Intel Mac OS X 10_15_7',
    scaleFactors: [2, 2, 1.5],
    resolutions: [
      { width: 2560, height: 1600 },
      { width: 2880, height: 1800 },
      { width: 1920, height: 1200 },
    ],
    webglVendor: 'Google Inc. (Apple)',
    webglRenderers: ['ANGLE (Apple, Apple M2, OpenGL 4.1)', 'ANGLE (Apple, Apple M1, OpenGL 4.1)'],
    fonts: ['Helvetica Neue', 'Arial', 'Times New Roman', 'PingFang SC'],
  },
]
