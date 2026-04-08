import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { BatchOperation } from '@/types/operation'

export const useOperationsStore = defineStore('operations', () => {
  const operations = ref<BatchOperation[]>([])
  const activeOperationId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  return { operations, activeOperationId, isLoading, error }
})
