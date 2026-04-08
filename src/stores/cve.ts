import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { CveAlert } from '@/types/cve'

export const useCveStore = defineStore('cve', () => {
  const alerts = ref<CveAlert[]>([])
  const isLoading = ref(false)
  const lastCheckedAt = ref<string | null>(null)
  const error = ref<string | null>(null)

  const criticalCount = computed(() =>
    alerts.value.filter(a => a.severity === 'critical' && a.status === 'new').length,
  )
  const highCount = computed(() =>
    alerts.value.filter(a => a.severity === 'high' && a.status === 'new').length,
  )
  const badgeCount = computed(() => criticalCount.value + highCount.value)

  return { alerts, isLoading, lastCheckedAt, error, criticalCount, highCount, badgeCount }
})
