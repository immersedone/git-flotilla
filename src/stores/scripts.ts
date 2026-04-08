import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScriptPreset, ScriptRun } from '@/types/script'

export const useScriptsStore = defineStore('scripts', () => {
  const presets = ref<ScriptPreset[]>([])
  const activeRun = ref<ScriptRun | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  return { presets, activeRun, isLoading, error }
})
