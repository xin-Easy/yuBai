import { tauriInvoke } from '@/utils/tauri'
import type {
  BrowserProxy,
  BrowserProxyBatchTestResult,
  BrowserProxyImportInput,
  BrowserProxyImportResult,
  BrowserProxyInput,
  BrowserProxySubscriptionFetchResult,
  BrowserProxyTestResult,
} from '@/types'

export async function fetchBrowserProxies(): Promise<BrowserProxy[]> {
  return tauriInvoke<BrowserProxy[]>('browser_proxy_list')
}

export async function saveBrowserProxy(input: BrowserProxyInput): Promise<BrowserProxy> {
  return tauriInvoke<BrowserProxy>('browser_proxy_save', { input })
}

export async function deleteBrowserProxy(proxyId: string): Promise<void> {
  return tauriInvoke<void>('browser_proxy_delete', { proxyId })
}

export async function importBrowserProxies(input: BrowserProxyImportInput): Promise<BrowserProxyImportResult> {
  return tauriInvoke<BrowserProxyImportResult>('browser_proxy_import_text', { input })
}

export async function fetchClashSubscription(url: string): Promise<BrowserProxySubscriptionFetchResult> {
  return tauriInvoke<BrowserProxySubscriptionFetchResult>('browser_proxy_fetch_clash_by_url', { url })
}

export async function testBrowserProxy(proxyId: string): Promise<BrowserProxyTestResult> {
  return tauriInvoke<BrowserProxyTestResult>('browser_proxy_test', { proxyId })
}

export async function batchTestBrowserProxies(proxyIds: string[]): Promise<BrowserProxyBatchTestResult> {
  return tauriInvoke<BrowserProxyBatchTestResult>('browser_proxy_batch_test', { proxyIds })
}

export async function checkBrowserProxyIPHealth(proxyId: string): Promise<BrowserProxyTestResult> {
  return tauriInvoke<BrowserProxyTestResult>('browser_proxy_ip_health', { proxyId })
}

export async function refreshBrowserProxySubscription(sourceId: string): Promise<BrowserProxyImportResult> {
  return tauriInvoke<BrowserProxyImportResult>('browser_proxy_refresh_subscription', { sourceId })
}

export async function listBrowserProxiesBySource(sourceId: string): Promise<BrowserProxy[]> {
  return tauriInvoke<BrowserProxy[]>('browser_proxy_list_by_source', { sourceId })
}
