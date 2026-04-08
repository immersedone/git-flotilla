import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DependencyMatrix } from '@/types/package'

export const usePackagesStore = defineStore('packages', () => {
  const matrix = ref<DependencyMatrix | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedEcosystem = ref<string | null>(null)

  return { matrix, isLoading, error, selectedEcosystem }
})
