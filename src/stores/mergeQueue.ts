import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { FlotillaPr } from '@/types/mergeQueue'
import {
  listFlotillaPrs,
  mergePr,
  mergeAllGreen,
} from '@/services/mergeQueue'

export const useMergeQueueStore = defineStore('mergeQueue', () => {
  const prs = ref<FlotillaPr[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const greenPrs = computed(() =>
    prs.value.filter(p => p.ciStatus === 'success' && p.mergeable === 'MERGEABLE'),
  )

  const prCount = computed(() => prs.value.length)

  async function loadPrs(operationId?: string) {
    isLoading.value = true
    error.value = null
    try {
      prs.value = await listFlotillaPrs(operationId)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function mergeSinglePr(repoId: string, prNumber: number) {
    error.value = null
    try {
      await mergePr(repoId, prNumber)
      await loadPrs()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function mergeAllGreenAction(operationId?: string) {
    error.value = null
    try {
      const count = await mergeAllGreen(operationId)
      await loadPrs()
      return count
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    prs,
    isLoading,
    error,
    greenPrs,
    prCount,
    loadPrs,
    mergeSinglePr,
    mergeAllGreenAction,
  }
})
