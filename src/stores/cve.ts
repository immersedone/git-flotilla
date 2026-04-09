import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { CveAlert, CveSeverity, CveStatus, BlastRadius } from '@/types/cve'
import {
  listCveAlerts,
  checkCves,
  acknowledgeCve,
  dismissCve,
  snoozeCve,
  getBlastRadius as fetchBlastRadius,
  listWatchlist as fetchWatchlist,
  addToWatchlist,
  removeFromWatchlist,
} from '@/services/cve'

interface WatchlistEntry {
  packageName: string
  ecosystem: string
  addedAt: string
}

export const useCveStore = defineStore('cve', () => {
  const alerts = ref<CveAlert[]>([])
  const isLoading = ref(false)
  const lastCheckedAt = ref<string | null>(null)
  const error = ref<string | null>(null)

  const watchlist = ref<WatchlistEntry[]>([])
  const selectedSeverity = ref<CveSeverity | null>(null)
  const selectedStatus = ref<CveStatus | null>(null)
  const blastRadius = ref<BlastRadius | null>(null)

  const filteredAlerts = computed(() => {
    return alerts.value.filter((a) => {
      if (selectedSeverity.value && a.severity !== selectedSeverity.value) return false
      if (selectedStatus.value && a.status !== selectedStatus.value) return false
      return true
    })
  })

  const criticalCount = computed(() =>
    alerts.value.filter(a => a.severity === 'critical' && a.status === 'new').length,
  )
  const highCount = computed(() =>
    alerts.value.filter(a => a.severity === 'high' && a.status === 'new').length,
  )
  const badgeCount = computed(() => criticalCount.value + highCount.value)

  async function loadAlerts(severity?: CveSeverity, status?: CveStatus): Promise<void> {
    error.value = null
    isLoading.value = true
    try {
      alerts.value = await listCveAlerts(severity, status)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function runCheck(): Promise<void> {
    error.value = null
    isLoading.value = true
    try {
      alerts.value = await checkCves()
      lastCheckedAt.value = new Date().toISOString()
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function acknowledgeAlert(cveId: string, repoId?: string): Promise<void> {
    error.value = null
    try {
      await acknowledgeCve(cveId, repoId)
      await loadAlerts()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function dismissAlert(cveId: string, repoId?: string): Promise<void> {
    error.value = null
    try {
      await dismissCve(cveId, repoId)
      await loadAlerts()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function snoozeAlert(cveId: string, repoId: string, days: number): Promise<void> {
    error.value = null
    try {
      await snoozeCve(cveId, repoId, days)
      await loadAlerts()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function loadWatchlist(): Promise<void> {
    error.value = null
    try {
      watchlist.value = await fetchWatchlist()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function addToWatchlistAction(packageName: string, ecosystem: string): Promise<void> {
    error.value = null
    try {
      await addToWatchlist(packageName, ecosystem)
      await loadWatchlist()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function removeFromWatchlistAction(packageName: string, ecosystem: string): Promise<void> {
    error.value = null
    try {
      await removeFromWatchlist(packageName, ecosystem)
      await loadWatchlist()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function loadBlastRadius(cveId: string): Promise<void> {
    error.value = null
    try {
      blastRadius.value = await fetchBlastRadius(cveId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    alerts,
    isLoading,
    lastCheckedAt,
    error,
    watchlist,
    selectedSeverity,
    selectedStatus,
    blastRadius,
    filteredAlerts,
    criticalCount,
    highCount,
    badgeCount,
    loadAlerts,
    runCheck,
    acknowledgeAlert,
    dismissAlert,
    snoozeAlert,
    loadWatchlist,
    addToWatchlistAction,
    removeFromWatchlistAction,
    loadBlastRadius,
  }
})
