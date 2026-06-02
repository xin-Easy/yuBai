import { tauriInvoke } from '@/utils/tauri'
import type { BookmarkSyncResult, BrowserBookmark } from '@/types'

export async function fetchBookmarks(): Promise<BrowserBookmark[]> {
  return tauriInvoke<BrowserBookmark[]>('bookmark_list')
}

export async function saveBookmarks(items: BrowserBookmark[]): Promise<void> {
  return tauriInvoke<void>('bookmark_save', { items })
}

export async function resetBookmarks(): Promise<void> {
  return tauriInvoke<void>('bookmark_reset')
}

export async function syncBookmarksToProfiles(): Promise<BookmarkSyncResult> {
  return tauriInvoke<BookmarkSyncResult>('bookmark_sync_to_profiles')
}
