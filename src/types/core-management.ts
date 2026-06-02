import type {
  BrowserCore,
  BrowserCoreDownloadProgress,
  BrowserCoreValidateResult,
} from './index'

export interface CoreRow extends BrowserCore {
  chromeVersion: string
  instanceCount: number
  pathValid: boolean
  pathMessage: string
}

export type EditorMode = 'create' | 'edit'
export type EditorSource = 'manual' | 'system' | 'download'
export type DownloadMode = 'official' | 'custom'

export interface DownloadFormState {
  mode: DownloadMode
  selectedUrl: string
  customUrl: string
  coreName: string
  proxyConfig: string
  isDefault: boolean
}

export interface SegmentedOption<T extends string = string> {
  label: string
  value: T
}

export interface DownloadSummary {
  platform: string
  version: string
  channel: string
}

export type PathValidationState = BrowserCoreValidateResult | null
export type DownloadProgressState = BrowserCoreDownloadProgress | null
