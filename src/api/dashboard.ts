import { tauriInvoke } from '@/utils/tauri'
import type { DashboardStats, DashboardWorkbench } from '@/types/dashboard'

export async function fetchDashboardStats(): Promise<DashboardStats> {
  return tauriInvoke<DashboardStats>('dashboard_stats_get')
}

export async function fetchDashboardWorkbench(): Promise<DashboardWorkbench> {
  return tauriInvoke<DashboardWorkbench>('dashboard_workbench_get')
}
