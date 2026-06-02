export interface BrowserFingerprint {
  browserBrand: string
  os: string
  country: string
  language: string
  userAgent: string
  acceptLanguage: string
  platform: string
  vendor: string
  timezone: string
  resolutionWidth: number
  resolutionHeight: number
  availWidth: number
  availHeight: number
  windowWidth: number
  windowHeight: number
  windowX: number
  windowY: number
  deviceScaleFactor: number
  colorDepth: number
  cpuCores: number
  gpuMemory: number
  diskType: string
  keyboardLayout: string
  webglRenderer: string
  webglVendor: string
  canvasPolicy: string
  webglPolicy: string
  webRtcPolicy: string
  geoLatitude: number
  geoLongitude: number
  geoAccuracy: number
  geoPolicy: string
  fontPolicy: string
  fontList: string[]
  microphoneCount: number
  speakerCount: number
  cameraCount: number
  startMaximized: boolean
  audioCodec: boolean
  camera: boolean
  doNotTrack: boolean
  uaRandom: boolean
  ldpEnabled: boolean
}

export interface BrowserProfile {
  profileId: string
  profileName: string
  userDataDir: string
  coreId: string
  fingerprintArgs: string[]
  fingerprint?: BrowserFingerprint
  proxyId: string
  proxyConfig: string
  proxyBindSourceId?: string
  proxyBindSourceUrl?: string
  proxyBindName?: string
  proxyBindUpdatedAt?: string
  launchArgs: string[]
  tags: string[]
  keywords: string[]
  groupId?: string
  running: boolean
  debugPort: number
  debugReady: boolean
  pid: number
  automationRunId?: string
  runtimeProxySummary: string
  runtimeWarning: string
  lastError: string
  createdAt: string
  updatedAt: string
  lastStartAt?: string
  lastStopAt?: string
  launchCode?: string
}

export interface BrowserProfileInput {
  profileName: string
  userDataDir: string
  coreId: string
  fingerprintArgs: string[]
  fingerprint?: BrowserFingerprint
  proxyId: string
  proxyConfig: string
  launchArgs: string[]
  tags: string[]
  keywords: string[]
  groupId?: string
}

export interface BrowserCore {
  coreId: string
  coreName: string
  corePath: string
  isDefault: boolean
}

export interface BrowserCoreInput {
  coreId?: string
  coreName: string
  corePath: string
  isDefault: boolean
}

export interface BrowserCoreValidateResult {
  valid: boolean
  message: string
  executablePath?: string
}

export interface BrowserCoreExtendedInfo {
  coreId: string
  chromeVersion: string
  instanceCount: number
  pathValid: boolean
  pathMessage: string
}

export interface BrowserCoreDownloadOption {
  version: string
  channel: string
  platform: string
  url: string
}

export interface BrowserCoreDownloadInput {
  coreName: string
  version: string
  url: string
  proxyConfig?: string
  isDefault: boolean
}

export interface BrowserCoreDownloadProgress {
  phase: string
  progress: number
  message: string
}

export interface SystemBrowserCandidate {
  name: string
  path: string
  version: string
  source: string
  registered: boolean
}

export interface BrowserProxy {
  proxyId: string
  proxyName: string
  proxyConfig: string
  dnsServers?: string
  groupName?: string
  sourceId?: string
  sourceUrl?: string
  sourceNamePrefix?: string
  sourceAutoRefresh: boolean
  sourceRefreshIntervalM: number
  sourceLastRefreshAt?: string
  lastLatencyMs: number
  lastTestOk: boolean
  lastTestedAt?: string
  lastIPHealthJson?: string
  boundProfileNames?: string[]
}

export interface BrowserProxyInput {
  proxyId?: string
  proxyName: string
  proxyConfig: string
  dnsServers?: string
  groupName?: string
  sourceId?: string
  sourceUrl?: string
  sourceNamePrefix?: string
}

export interface BrowserProxyImportInput {
  content: string
  groupName?: string
  namePrefix?: string
  dnsServers?: string
  sourceId?: string
  sourceUrl?: string
  sourceNamePrefix?: string
}

export interface BrowserProxySubscriptionFetchResult {
  url: string
  content: string
  proxyCount: number
  dnsServers?: string
  suggestedGroup?: string
}

export interface BrowserProxyImportResult {
  imported: number
  skipped: number
  failed: number
  items: BrowserProxy[]
  errors: string[]
}

export interface BrowserProxyTestResult {
  proxyId: string
  ok: boolean
  latencyMs: number
  testedAt: string
  ipHealthJson?: string
  error?: string
}

export interface BrowserProxyBatchTestResult {
  total: number
  ok: number
  failed: number
  items: BrowserProxyTestResult[]
}

export interface BrowserTabInfo {
  targetId: string
  targetType: string
  title: string
  url: string
  attached: boolean
}

export interface BrowserGroup {
  groupId: string
  groupName: string
  parentId: string
  color?: string
  sortOrder: number
  createdAt: string
  updatedAt: string
}

export interface BrowserGroupInput {
  groupId?: string
  groupName: string
  parentId?: string
  color?: string
  sortOrder: number
}

export interface BrowserGroupWithCount extends BrowserGroup {
  profileCount: number
  instanceCount: number
}

export interface BrowserBookmark {
  name: string
  url: string
  openOnStart: boolean
}

export interface BookmarkSyncResult {
  total: number
  synced: number
  skipped: number
  failed: number
  skippedList: string[]
  failedList: string[]
}

export interface BrowserSettings {
  defaultFingerprintArgs: string[]
  defaultLaunchArgs: string[]
  defaultStartUrls: string[]
  downloadSource: 'auto' | 'official' | 'npmmirror'
  mihomoDownloadSource: 'auto' | 'official' | 'npmmirror'
  proxyMode: 'auto' | 'mihomo' | 'extension'
  mihomoDownloaded: boolean
  mihomoBinaryPath: string
  restoreLastSession: boolean
  startReadyTimeoutMs: number
  startStableWindowMs: number
  userDataRoot: string
}

export interface AppLogEntry {
  level: string
  target: string
  message: string
  details: string
  createdAt: string
}

export interface SnapshotInfo {
  snapshotId: string
  profileId: string
  name: string
  sizeMB: number
  createdAt: string
}

export interface CookieInfo {
  name: string
  value: string
  domain: string
  path: string
  expires: number
  httpOnly: boolean
  secure: boolean
  sameSite: string
}
