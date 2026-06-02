import { tauriInvoke } from '@/utils/tauri'
import type {
  AutomationRun,
  AutomationRunInput,
  AutomationScript,
  AutomationScriptInput,
  AutomationSelfCheck,
  AutomationState,
} from '@/types/automation'

export async function fetchAutomationState(): Promise<AutomationState> {
  return tauriInvoke<AutomationState>('automation_state_get')
}

export async function saveAutomationSettings(enabled: boolean, headlessDefault: boolean): Promise<AutomationState> {
  return tauriInvoke<AutomationState>('automation_settings_save', { enabled, headlessDefault })
}

export async function installAutomationRuntime(): Promise<AutomationState> {
  return tauriInvoke<AutomationState>('automation_runtime_install')
}

export async function selfCheckAutomationRuntime(): Promise<AutomationSelfCheck> {
  return tauriInvoke<AutomationSelfCheck>('automation_runtime_self_check')
}

export async function fetchAutomationScripts(): Promise<AutomationScript[]> {
  return tauriInvoke<AutomationScript[]>('automation_script_list')
}

export async function createAutomationScript(input: AutomationScriptInput): Promise<AutomationScript> {
  return tauriInvoke<AutomationScript>('automation_script_create', { input })
}

export async function updateAutomationScript(scriptId: string, input: AutomationScriptInput): Promise<AutomationScript> {
  return tauriInvoke<AutomationScript>('automation_script_update', { scriptId, input })
}

export async function deleteAutomationScript(scriptId: string): Promise<void> {
  return tauriInvoke<void>('automation_script_delete', { scriptId })
}

export async function runAutomationScript(input: AutomationRunInput): Promise<AutomationRun> {
  return tauriInvoke<AutomationRun>('automation_script_run', { input })
}

export async function cancelAutomationRun(runId: string): Promise<AutomationRun> {
  return tauriInvoke<AutomationRun>('automation_script_run_cancel', { runId })
}

export async function fetchAutomationScriptRuns(limit = 20): Promise<AutomationRun[]> {
  return tauriInvoke<AutomationRun[]>('automation_script_run_list', { limit })
}
