import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { BatchOperation, CreateOperationInput } from '@/types/operation'
import type { OperationProgressEvent } from '@/services/operations'
import {
  listOperations,
  createOperation,
  runOperation,
  abortOperation,
  getOperation,
  rollbackOperation,
  onOperationProgress,
} from '@/services/operations'
import type { UnlistenFn } from '@tauri-apps/api/event'

export const useOperationsStore = defineStore('operations', () => {
  const operations = ref<BatchOperation[]>([])
  const activeOperation = ref<BatchOperation | null>(null)
  const isLoading = ref(false)
  const isRunning = ref(false)
  const error = ref<string | null>(null)
  const progress = ref({ current: 0, total: 0 })
  const repoStatuses = ref<Record<string, OperationProgressEvent>>({})

  const pendingOps = computed(() =>
    operations.value.filter((op) => op.status === 'pending'),
  )

  const runningOps = computed(() =>
    operations.value.filter((op) => op.status === 'running'),
  )

  const completedOps = computed(() =>
    operations.value.filter((op) => op.status === 'completed'),
  )

  async function loadOperations() {
    isLoading.value = true
    error.value = null
    try {
      operations.value = await listOperations()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createOp(input: CreateOperationInput) {
    error.value = null
    try {
      const op = await createOperation(input)
      operations.value.unshift(op)
      return op
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function runOp(id: string) {
    error.value = null
    isRunning.value = true
    progress.value = { current: 0, total: 0 }
    repoStatuses.value = {}

    let unlisten: UnlistenFn | null = null
    try {
      unlisten = await onOperationProgress((event) => {
        if (event.operationId === id) {
          progress.value = { current: event.current, total: event.total }
          repoStatuses.value[event.repoId] = event

          // Update the operation status in the list
          const idx = operations.value.findIndex((op) => op.id === id)
          if (idx !== -1) {
            operations.value[idx] = {
              ...operations.value[idx],
              status: 'running',
            }
          }
        }
      })

      await runOperation(id)
      await loadOperations()

      // Refresh activeOperation if it matches
      if (activeOperation.value?.id === id) {
        activeOperation.value =
          operations.value.find((op) => op.id === id) ?? null
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      isRunning.value = false
      if (unlisten) {
        unlisten()
      }
    }
  }

  async function abortOp(id: string) {
    error.value = null
    try {
      await abortOperation(id)
      await loadOperations()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function getOp(id: string) {
    error.value = null
    try {
      activeOperation.value = await getOperation(id)
      return activeOperation.value
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function rollbackOp(id: string) {
    error.value = null
    try {
      await rollbackOperation(id)
      await loadOperations()
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    operations,
    activeOperation,
    isLoading,
    isRunning,
    error,
    progress,
    repoStatuses,
    pendingOps,
    runningOps,
    completedOps,
    loadOperations,
    createOp,
    runOp,
    abortOp,
    getOp,
    rollbackOp,
  }
})
