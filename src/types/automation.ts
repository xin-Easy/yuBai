export interface AutomationState {
  enabled: boolean
  runtimeVersion: string
  headlessDefault: boolean
  installed: boolean
  ready: boolean
  installing: boolean
  lastError: string
  nodeVersion: string
  playwrightVersion: string
}

export interface AutomationSelfCheck {
  ok: boolean
  nodeSource: string
  nodeVersion: string
  playwrightVersion: string
  runnerPath: string
  error: string
}

export interface AutomationScript {
  scriptId: string
  name: string
  description: string
  entryFile: string
  version: string
  createdAt: string
  updatedAt: string
  content: string
}

export interface AutomationScriptInput {
  name: string
  description?: string
  content: string
}

export interface AutomationRun {
  runId: string
  scriptId: string
  scriptName: string
  profileId: string
  profileName: string
  status: 'pending' | 'running' | 'success' | 'failed' | 'cancelled'
  startedAt: string
  finishedAt: string
  durationMs: number
  exitCode: number | null
  stdout: string
  stderr: string
  resultJson: string
  error: string
}

export interface AutomationRunInput {
  scriptId: string
  profileId: string
  params?: Record<string, unknown>
  timeoutMs?: number
}

export interface AutomationRunEvent {
  runId: string
  status: AutomationRun['status']
  message: string
  run?: AutomationRun
}

export interface AutomationRunLogEvent {
  runId: string
  stream: 'stdout' | 'stderr'
  line: string
  createdAt: string
}

export interface AutomationRuntimeProgressEvent {
  phase: string
  message: string
  progress: number
}
