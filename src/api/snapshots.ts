import { tauriInvoke } from '@/utils/tauri'
import type { SnapshotInfo } from '@/types'

export async function listSnapshots(profileId: string): Promise<SnapshotInfo[]> {
  return tauriInvoke<SnapshotInfo[]>('browser_snapshot_list', { profileId })
}

export async function createSnapshot(profileId: string, name: string): Promise<SnapshotInfo> {
  return tauriInvoke<SnapshotInfo>('browser_snapshot_create', { profileId, name })
}

export async function restoreSnapshot(profileId: string, snapshotId: string): Promise<void> {
  return tauriInvoke<void>('browser_snapshot_restore', { profileId, snapshotId })
}

export async function deleteSnapshot(profileId: string, snapshotId: string): Promise<void> {
  return tauriInvoke<void>('browser_snapshot_delete', { profileId, snapshotId })
}
