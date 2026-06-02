import { tauriInvoke } from '@/utils/tauri'
import type {
  BrowserCore,
  BrowserCoreDownloadInput,
  BrowserCoreDownloadOption,
  BrowserCoreExtendedInfo,
  BrowserCoreInput,
  BrowserCoreValidateResult,
  SystemBrowserCandidate,
} from '@/types'

export async function fetchBrowserCores(): Promise<BrowserCore[]> {
  return tauriInvoke<BrowserCore[]>('browser_core_list')
}

export async function saveBrowserCore(input: BrowserCoreInput): Promise<BrowserCore> {
  return tauriInvoke<BrowserCore>('browser_core_save', { input })
}

export async function deleteBrowserCore(coreId: string): Promise<void> {
  return tauriInvoke<void>('browser_core_delete', { coreId })
}

export async function validateBrowserCore(corePath: string): Promise<BrowserCoreValidateResult> {
  return tauriInvoke<BrowserCoreValidateResult>('browser_core_validate', { corePath })
}

export async function fetchBrowserCoreExtendedInfo(): Promise<BrowserCoreExtendedInfo[]> {
  return tauriInvoke<BrowserCoreExtendedInfo[]>('browser_core_extended_info')
}

export async function scanLocalBrowserCores(): Promise<BrowserCore[]> {
  return tauriInvoke<BrowserCore[]>('browser_core_scan_local')
}

export async function detectSystemBrowsers(): Promise<SystemBrowserCandidate[]> {
  return tauriInvoke<SystemBrowserCandidate[]>('browser_core_detect_system')
}

export async function registerSystemBrowser(
  path: string,
  coreName?: string,
  isDefault = false,
): Promise<BrowserCore> {
  return tauriInvoke<BrowserCore>('browser_core_register_system', { path, coreName, isDefault })
}

export async function fetchBrowserCoreDownloadOptions(limit = 24): Promise<BrowserCoreDownloadOption[]> {
  return tauriInvoke<BrowserCoreDownloadOption[]>('browser_core_download_options', { limit })
}

export async function downloadBrowserCore(input: BrowserCoreDownloadInput): Promise<void> {
  return tauriInvoke<void>('browser_core_download', { input })
}
