import { tauriInvoke } from '@/utils/tauri'
import type { BrowserGroup, BrowserGroupInput, BrowserGroupWithCount } from '@/types'

export async function fetchBrowserGroups(): Promise<BrowserGroupWithCount[]> {
  return tauriInvoke<BrowserGroupWithCount[]>('browser_group_list')
}

export async function saveBrowserGroup(input: BrowserGroupInput): Promise<BrowserGroup> {
  return tauriInvoke<BrowserGroup>('browser_group_save', { input })
}

export async function deleteBrowserGroup(groupId: string): Promise<void> {
  return tauriInvoke<void>('browser_group_delete', { groupId })
}

export async function moveProfilesToGroup(profileIds: string[], groupId: string): Promise<void> {
  return tauriInvoke<void>('browser_group_move_profiles', { input: { profileIds, groupId } })
}
