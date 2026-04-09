import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScriptPreset, ScriptRun } from '@/types/script'
import {
  listPresets,
  savePreset,
  deletePreset,
  runScript,
  getScriptRun,
  abortScript,
} from '@/services/scripts'

export const useScriptsStore = defineStore('scripts', () => {
  const presets = ref<ScriptPreset[]>([])
  const activeRun = ref<ScriptRun | null>(null)
  const isRunning = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function loadPresets() {
    isLoading.value = true
    error.value = null
    try {
      presets.value = await listPresets()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function savePresetAction(preset: ScriptPreset) {
    error.value = null
    try {
      const saved = await savePreset(preset)
      const idx = presets.value.findIndex(p => p.id === saved.id)
      if (idx !== -1) {
        presets.value[idx] = saved
      } else {
        presets.value.push(saved)
      }
      return saved
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function deletePresetAction(id: string) {
    error.value = null
    try {
      await deletePreset(id)
      presets.value = presets.value.filter(p => p.id !== id)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function runScriptAction(command: string, repoIds: string[], parallel: number = 5) {
    error.value = null
    isRunning.value = true
    activeRun.value = null
    try {
      const runId = await runScript(command, repoIds, parallel)
      const run = await getScriptRun(runId)
      activeRun.value = run
      return run
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isRunning.value = false
    }
  }

  async function abortScriptAction() {
    error.value = null
    try {
      if (activeRun.value) {
        await abortScript(activeRun.value.id)
        activeRun.value = { ...activeRun.value, status: 'aborted' }
      }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    presets,
    activeRun,
    isRunning,
    isLoading,
    error,
    loadPresets,
    savePresetAction,
    deletePresetAction,
    runScriptAction,
    abortScriptAction,
  }
})
