import { tauriInvoke } from '@/utils/tauri'
import type { AppLogEntry, BrowserSettings } from '@/types'

export async function fetchBrowserSettings(): Promise<BrowserSettings> {
  return tauriInvoke<BrowserSettings>('browser_settings_get')
}

export async function saveBrowserSettings(settings: BrowserSettings): Promise<void> {
  return tauriInvoke<void>('browser_settings_save', { settings })
}

export async function fetchAppLogs(): Promise<AppLogEntry[]> {
  return tauriInvoke<AppLogEntry[]>('app_logs_get')
}

export async function clearAppLogs(): Promise<void> {
  return tauriInvoke<void>('app_logs_clear')
}

export async function downloadMihomo(): Promise<string> {
  return tauriInvoke<string>('mihomo_download')
}
