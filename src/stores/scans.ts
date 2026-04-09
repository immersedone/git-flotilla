import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ScanResult, ScanProgressEvent, BatchScanSummary } from '@/types/scan'
import type { UnlistenFn } from '@tauri-apps/api/event'
import {
  scanRepo,
  scanRepoList,
  listScanResults,
  abortScan,
  onScanProgress,
} from '@/services/scan'

export const useScansStore = defineStore('scans', () => {
  const results = ref<ScanResult[]>([])
  const isScanning = ref(false)
  const scanProgress = ref({ current: 0, total: 0 })
  const error = ref<string | null>(null)
  const repoStatuses = ref<Record<string, ScanProgressEvent>>({})

  let unlistenProgress: UnlistenFn | null = null
  let currentOperationId: string | null = null

  const averageHealthScore = computed(() => {
    if (results.value.length === 0) return 0
    return Math.round(
      results.value.reduce((sum, r) => sum + r.healthScore, 0) / results.value.length,
    )
  })

  const scanSummary = computed<BatchScanSummary>(() => {
    const statuses = Object.values(repoStatuses.value)
    return {
      total: statuses.length,
      succeeded: statuses.filter(s => s.status === 'done').length,
      failed: statuses.filter(s => s.status === 'failed').length,
      inProgress: statuses.filter(s => s.status === 'scanning').length,
    }
  })

  async function loadResults(repoListId?: string) {
    error.value = null
    try {
      results.value = await listScanResults(repoListId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function scanSingleRepo(repoId: string) {
    error.value = null
    try {
      const result = await scanRepo(repoId)
      const idx = results.value.findIndex(r => r.repoId === repoId)
      if (idx !== -1) {
        results.value[idx] = result
      } else {
        results.value.push(result)
      }
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function scanList(listId: string) {
    error.value = null
    isScanning.value = true
    scanProgress.value = { current: 0, total: 0 }
    repoStatuses.value = {}

    try {
      unlistenProgress = await onScanProgress((event: ScanProgressEvent) => {
        repoStatuses.value[event.repoId] = event
        scanProgress.value = { current: event.current, total: event.total }
      })

      currentOperationId = await scanRepoList(listId)
      await loadResults(listId)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isScanning.value = false
      if (unlistenProgress) {
        unlistenProgress()
        unlistenProgress = null
      }
      currentOperationId = null
    }
  }

  async function abortCurrentScan() {
    error.value = null
    try {
      if (currentOperationId) {
        await abortScan(currentOperationId)
      }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    results,
    isScanning,
    scanProgress,
    error,
    repoStatuses,
    averageHealthScore,
    scanSummary,
    loadResults,
    scanSingleRepo,
    scanList,
    abortCurrentScan,
  }
})
