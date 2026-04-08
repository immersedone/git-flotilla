import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ScanResult } from '@/types/scan'

export const useScansStore = defineStore('scans', () => {
  const results = ref<ScanResult[]>([])
  const isScanning = ref(false)
  const scanProgress = ref({ current: 0, total: 0 })
  const error = ref<string | null>(null)

  const averageHealthScore = computed(() => {
    if (results.value.length === 0) return 0
    return Math.round(results.value.reduce((sum, r) => sum + r.healthScore, 0) / results.value.length)
  })

  return { results, isScanning, scanProgress, error, averageHealthScore }
})
