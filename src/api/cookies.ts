import { tauriInvoke } from '@/utils/tauri'
import type { CookieInfo } from '@/types'

export async function fetchBrowserCookies(profileId: string): Promise<CookieInfo[]> {
  return tauriInvoke<CookieInfo[]>('browser_get_cookies', { profileId })
}

export async function clearBrowserCookies(profileId: string): Promise<void> {
  return tauriInvoke<void>('browser_clear_cookies', { profileId })
}

export async function exportBrowserCookies(profileId: string): Promise<string> {
  return tauriInvoke<string>('browser_export_cookies', { profileId })
}
