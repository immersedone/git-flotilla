import { invoke } from '@tauri-apps/api/core'
import type { ScriptPreset, ScriptRun } from '@/types/script'

export function runScript(command: string, repoIds: string[], parallel: number): Promise<string> {
  return invoke('run_script', { command, repoIds, parallel })
}

export function getScriptRun(runId: string): Promise<ScriptRun> {
  return invoke('get_script_run', { runId })
}

export function abortScript(runId: string): Promise<void> {
  return invoke('abort_script', { runId })
}

export function listPresets(): Promise<ScriptPreset[]> {
  return invoke('list_presets')
}

export function savePreset(preset: ScriptPreset): Promise<ScriptPreset> {
  return invoke('save_preset', { preset })
}

export function deletePreset(id: string): Promise<void> {
  return invoke('delete_preset', { id })
}
