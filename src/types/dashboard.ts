export interface DashboardStats {
  totalInstances: number
  runningInstances: number
  proxyCount: number
  coreCount: number
  appVersion: string
  os: string
  arch: string
  appRoot: string
}

export interface DashboardWorkbench {
  stats: DashboardWorkbenchStats
  system: DashboardWorkbenchSystem
  defaultCore?: DashboardWorkbenchDefaultCore | null
  automation: DashboardWorkbenchAutomation
  runningProfiles: DashboardWorkbenchRunningProfile[]
}

export interface DashboardWorkbenchStats {
  totalInstances: number
  runningInstances: number
  unboundInstances: number
  proxyCount: number
  healthyProxyCount: number
  untestedProxyCount: number
  coreCount: number
  invalidCoreCount: number
  scriptCount: number
  runCount: number
}

export interface DashboardWorkbenchSystem {
  appVersion: string
  os: string
  arch: string
  appRoot: string
  proxyMode: string
  mihomoDownloaded: boolean
}

export interface DashboardWorkbenchDefaultCore {
  coreId: string
  coreName: string
  pathValid: boolean
  pathMessage: string
}

export interface DashboardWorkbenchAutomation {
  ready: boolean
  installed: boolean
  lastError: string
  nodeVersion: string
  playwrightVersion: string
}

export interface DashboardWorkbenchRunningProfile {
  profileId: string
  profileName: string
  debugPort: number
  runtimeProxySummary: string
  updatedAt: string
}
