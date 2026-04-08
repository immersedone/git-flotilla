import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { FlotillaPr } from '@/types/mergeQueue'

export const useMergeQueueStore = defineStore('mergeQueue', () => {
  const prs = ref<FlotillaPr[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const greenPrs = computed(() =>
    prs.value.filter(p => p.ciStatus === 'success' && p.mergeable === 'MERGEABLE'),
  )

  return { prs, isLoading, error, greenPrs }
})
