import { invoke } from '@tauri-apps/api/core'

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

function mockInvoke<T>(command: string, args?: Record<string, unknown>): T {
  switch (command) {
    case 'dashboard_stats_get':
      return {
        totalInstances: 0,
        runningInstances: 0,
        proxyCount: 0,
        coreCount: 0,
        appVersion: 'web-preview',
        os: 'web-preview',
        arch: 'web-preview',
        appRoot: '',
      } as T
    case 'dashboard_workbench_get':
      return {
        stats: {
          totalInstances: 0,
          runningInstances: 0,
          unboundInstances: 0,
          proxyCount: 0,
          healthyProxyCount: 0,
          untestedProxyCount: 0,
          coreCount: 0,
          invalidCoreCount: 0,
          scriptCount: 0,
          runCount: 0,
        },
        system: {
          appVersion: 'web-preview',
          os: 'web-preview',
          arch: 'web-preview',
          appRoot: '',
          proxyMode: 'auto',
          mihomoDownloaded: false,
        },
        defaultCore: null,
        automation: {
          ready: true,
          installed: true,
          lastError: '',
          nodeVersion: '22.15.1',
          playwrightVersion: '1.59.0',
        },
        runningProfiles: [],
      } as T
    case 'browser_profile_list':
      return [] as T
    case 'browser_profile_create': {
      const input = (args?.input ?? {}) as Record<string, unknown>
      return {
        profileId: `web-profile-${Date.now()}`,
        profileName: String(input.profileName ?? 'Web Preview Profile'),
        userDataDir: String(input.userDataDir ?? ''),
        coreId: String(input.coreId ?? ''),
        fingerprintArgs: input.fingerprintArgs ?? [],
        fingerprint: input.fingerprint,
        proxyId: String(input.proxyId ?? ''),
        proxyConfig: String(input.proxyConfig ?? ''),
        launchArgs: input.launchArgs ?? [],
        tags: input.tags ?? [],
        keywords: input.keywords ?? [],
        groupId: input.groupId,
        running: false,
        debugPort: 0,
        debugReady: false,
        pid: 0,
        runtimeProxySummary: '未运行',
        runtimeWarning: '',
        lastError: '',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      } as T
    }
    case 'browser_profile_delete':
      return undefined as T
    case 'browser_instance_start':
    case 'browser_instance_stop':
    case 'browser_instance_status':
      throw new Error('Web 预览环境不能控制本地浏览器进程，请在 Tauri 中运行')
    case 'browser_core_list':
      return [] as T
    case 'browser_core_save': {
      const input = (args?.input ?? {}) as Record<string, unknown>
      return {
        coreId: String(input.coreId ?? `web-core-${Date.now()}`),
        coreName: String(input.coreName ?? 'Web Preview Core'),
        corePath: String(input.corePath ?? ''),
        isDefault: Boolean(input.isDefault),
      } as T
    }
    case 'browser_core_delete':
      return undefined as T
    case 'browser_core_validate':
      return {
        valid: true,
        message: 'Web 预览环境跳过了本地路径校验',
        executablePath: String(args?.corePath ?? ''),
      } as T
    case 'browser_core_extended_info':
      return [] as T
    case 'browser_core_scan_local':
      return [] as T
    case 'browser_core_detect_system':
      return [] as T
    case 'browser_core_register_system':
      return {
        coreId: `system-core-${Date.now()}`,
        coreName: String(args?.coreName ?? 'System Chrome'),
        corePath: String(args?.path ?? ''),
        isDefault: Boolean(args?.isDefault),
      } as T
    case 'browser_core_download_options':
      return [
        {
          version: 'web-preview',
          channel: 'mock',
          platform: 'mock',
          url: 'https://example.com/chrome.zip',
        },
      ] as T
    case 'browser_core_download':
      return undefined as T
    case 'browser_proxy_list':
      return [
        {
          proxyId: 'proxy-direct',
          proxyName: '本机直连（不走代理）',
          proxyConfig: 'direct://',
          sourceAutoRefresh: false,
          sourceRefreshIntervalM: 0,
          lastLatencyMs: -1,
          lastTestOk: false,
          boundProfileNames: [],
        },
      ] as T
    case 'browser_proxy_save': {
      const input = (args?.input ?? {}) as Record<string, unknown>
      return {
        proxyId: String(input.proxyId ?? `web-proxy-${Date.now()}`),
        proxyName: String(input.proxyName ?? 'Web Preview Proxy'),
        proxyConfig: String(input.proxyConfig ?? ''),
        dnsServers: input.dnsServers,
        groupName: input.groupName,
        sourceAutoRefresh: false,
        sourceRefreshIntervalM: 0,
        lastLatencyMs: -1,
        lastTestOk: false,
      } as T
    }
    case 'browser_proxy_delete':
      return undefined as T
    case 'browser_proxy_import_text':
      return { imported: 0, skipped: 0, failed: 0, items: [], errors: [] } as T
    case 'browser_proxy_fetch_clash_by_url':
      return {
        url: String(args?.url ?? ''),
        content: '',
        proxyCount: 0,
        dnsServers: '',
        suggestedGroup: '',
      } as T
    case 'browser_proxy_test':
    case 'browser_proxy_ip_health':
      return {
        proxyId: String(args?.proxyId ?? ''),
        ok: false,
        latencyMs: -1,
        testedAt: new Date().toISOString(),
        error: 'Web 预览环境不能测试本地代理',
      } as T
    case 'browser_proxy_batch_test':
      return { total: 0, ok: 0, failed: 0, items: [] } as T
    case 'browser_group_list':
      return [] as T
    case 'browser_group_save': {
      const input = (args?.input ?? {}) as Record<string, unknown>
      const now = new Date().toISOString()
      return {
        groupId: String(input.groupId ?? `web-group-${Date.now()}`),
        groupName: String(input.groupName ?? 'Web Preview Group'),
        parentId: String(input.parentId ?? ''),
        sortOrder: Number(input.sortOrder ?? 0),
        createdAt: now,
        updatedAt: now,
      } as T
    }
    case 'browser_group_delete':
    case 'browser_group_move_profiles':
      return undefined as T
    case 'bookmark_list':
      return [] as T
    case 'bookmark_save':
    case 'bookmark_reset':
      return undefined as T
    case 'bookmark_sync_to_profiles':
      return { total: 0, synced: 0, skipped: 0, failed: 0, skippedList: [], failedList: [] } as T
    case 'browser_settings_get':
      return {
        userDataRoot: 'data/profiles',
        defaultFingerprintArgs: ['--fingerprint-brand=Chrome', '--fingerprint-platform=windows'],
        defaultLaunchArgs: ['--disable-sync', '--no-first-run'],
        defaultStartUrls: [],
        restoreLastSession: false,
        startReadyTimeoutMs: 3000,
        startStableWindowMs: 1200,
        downloadSource: 'auto',
        mihomoDownloadSource: 'auto',
        proxyMode: 'auto',
        mihomoDownloaded: false,
        mihomoBinaryPath: '',
      } as T
    case 'browser_settings_save':
    case 'app_logs_clear':
      return undefined as T
    case 'app_logs_get':
      return [] as T
    case 'browser_snapshot_list':
      return [] as T
    case 'browser_snapshot_create':
      return {
        snapshotId: `web-snapshot-${Date.now()}`,
        profileId: String(args?.profileId ?? ''),
        name: String(args?.name ?? 'Snapshot'),
        sizeMB: 0,
        createdAt: new Date().toISOString(),
      } as T
    case 'browser_snapshot_restore':
    case 'browser_snapshot_delete':
    case 'browser_clear_cookies':
      return undefined as T
    case 'browser_get_cookies':
      return [] as T
    case 'browser_export_cookies':
      return '[]' as T
    case 'automation_state_get':
      return {
        enabled: true,
        runtimeVersion: 'node-22.15.1-playwright-core-1.59.0',
        headlessDefault: false,
        installed: true,
        ready: true,
        installing: false,
        lastError: '',
        nodeVersion: '22.15.1',
        playwrightVersion: '1.59.0',
      } as T
    case 'automation_settings_save':
      return {
        enabled: Boolean(args?.enabled),
        runtimeVersion: 'node-22.15.1-playwright-core-1.59.0',
        headlessDefault: Boolean(args?.headlessDefault),
        installed: true,
        ready: Boolean(args?.enabled),
        installing: false,
        lastError: Boolean(args?.enabled) ? '' : 'automation runtime is disabled',
        nodeVersion: '22.15.1',
        playwrightVersion: '1.59.0',
      } as T
    case 'automation_runtime_self_check':
      return {
        ok: true,
        nodeSource: 'web-preview',
        nodeVersion: '22.15.1',
        playwrightVersion: '1.59.0',
        runnerPath: 'web-preview',
        error: '',
      } as T
    case 'automation_script_list':
      return [
        {
          scriptId: 'script-web-preview',
          name: 'Web 预览示例',
          description: '打开示例页面并读取标题',
          entryFile: 'main.js',
          version: '1.0.0',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          content: `async function run(ctx) {
  const page = ctx.page
  await page.goto('https://example.com')
  return { title: await page.title() }
}

module.exports = { run }
`,
        },
      ] as T
    case 'automation_script_run_list':
      return [] as T
    case 'automation_script_create': {
      const input = (args?.input ?? {}) as Record<string, unknown>
      return {
        scriptId: `script-web-${Date.now()}`,
        name: String(input.name ?? 'Web Preview Script'),
        description: String(input.description ?? ''),
        entryFile: 'main.js',
        version: '1.0.0',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        content: String(input.content ?? ''),
      } as T
    }
    case 'automation_script_update': {
      const input = (args?.input ?? {}) as Record<string, unknown>
      return {
        scriptId: String(args?.scriptId ?? `script-web-${Date.now()}`),
        name: String(input.name ?? 'Web Preview Script'),
        description: String(input.description ?? ''),
        entryFile: 'main.js',
        version: '1.0.0',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        content: String(input.content ?? ''),
      } as T
    }
    case 'automation_script_delete':
      return undefined as T
    case 'automation_script_run':
      return {
        runId: `run-web-${Date.now()}`,
        scriptId: String((args?.input as Record<string, unknown> | undefined)?.scriptId ?? ''),
        scriptName: 'Web 预览示例',
        profileId: String((args?.input as Record<string, unknown> | undefined)?.profileId ?? ''),
        profileName: 'Web Preview Profile',
        status: 'success',
        startedAt: new Date().toISOString(),
        finishedAt: new Date().toISOString(),
        durationMs: 128,
        exitCode: 0,
        stdout: '{"ok":true,"result":{"title":"Example Domain"}}',
        stderr: '',
        resultJson: '{"ok":true,"result":{"title":"Example Domain"}}',
        error: '',
      } as T
    case 'automation_script_run_cancel':
      return {
        runId: String(args?.runId ?? ''),
        scriptId: '',
        scriptName: 'Web 预览示例',
        profileId: '',
        profileName: 'Web Preview Profile',
        status: 'cancelled',
        startedAt: new Date().toISOString(),
        finishedAt: new Date().toISOString(),
        durationMs: 0,
        exitCode: null,
        stdout: '',
        stderr: '',
        resultJson: '',
        error: 'cancelled by user',
      } as T
    case 'automation_runtime_install':
      return {
        enabled: true,
        runtimeVersion: 'node-22.15.1-playwright-core-1.59.0',
        headlessDefault: false,
        installed: true,
        ready: true,
        installing: false,
        lastError: '',
        nodeVersion: '22.15.1',
        playwrightVersion: '1.59.0',
      } as T
    default:
      throw new Error(`当前是 Web 预览环境，命令 ${command} 需要在 Tauri 运行时中调用`)
  }
}

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    return mockInvoke<T>(command, args)
  }

  try {
    return await invoke<T>(command, args)
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    throw new Error(message || `调用 ${command} 失败`)
  }
}
