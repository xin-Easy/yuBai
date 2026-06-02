import { tauriInvoke } from '@/utils/tauri'
import type { BrowserProfile, BrowserProfileInput } from '@/types'

export async function fetchBrowserProfiles(): Promise<BrowserProfile[]> {
  return tauriInvoke<BrowserProfile[]>('browser_profile_list')
}

export async function createBrowserProfile(input: BrowserProfileInput): Promise<BrowserProfile> {
  return tauriInvoke<BrowserProfile>('browser_profile_create', { input })
}

export async function updateBrowserProfile(profileId: string, input: BrowserProfileInput): Promise<BrowserProfile> {
  return tauriInvoke<BrowserProfile>('browser_profile_update', { profileId, input })
}

export async function deleteBrowserProfile(profileId: string): Promise<void> {
  return tauriInvoke<void>('browser_profile_delete', { profileId })
}

export async function startBrowserInstance(profileId: string): Promise<BrowserProfile> {
  return tauriInvoke<BrowserProfile>('browser_instance_start', { profileId })
}

export async function stopBrowserInstance(profileId: string): Promise<BrowserProfile> {
  return tauriInvoke<BrowserProfile>('browser_instance_stop', { profileId })
}

export async function getBrowserInstanceStatus(profileId: string): Promise<BrowserProfile> {
  return tauriInvoke<BrowserProfile>('browser_instance_status', { profileId })
}
