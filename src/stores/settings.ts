import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  getRateLimitStatus,
  getSettings,
  saveSettings,
  listAuditLog,
} from '@/services/settings'
import type { RateLimitInfo } from '@/types/settings'

export type { RateLimitInfo }

export const useSettingsStore = defineStore('settings', () => {
  const scanIntervalMinutes    = ref<number | null>(1440)
  const cvePollIntervalMinutes = ref<number | null>(60)
  const parallelWorkers        = ref(5)
  const requestDelayMs         = ref(200)
  const darkMode               = ref(true)
  const rateLimitGithub        = ref<RateLimitInfo | null>(null)
  const rateLimitGitlab        = ref<RateLimitInfo | null>(null)

  const settings = ref<Record<string, string>>({})
  const auditLog = ref<Array<Record<string, unknown>>>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function refreshRateLimit() {
    try {
      const status = await getRateLimitStatus()
      rateLimitGithub.value = status.github
      rateLimitGitlab.value = status.gitlab
    } catch {
      // non-fatal — rate limit display is informational
    }
  }

  async function loadSettings() {
    isLoading.value = true
    error.value = null
    try {
      settings.value = await getSettings()
      // Sync convenience refs from loaded settings
      if (settings.value.scan_interval) {
        scanIntervalMinutes.value = Number(settings.value.scan_interval) || 1440
      }
      if (settings.value.cve_poll_interval) {
        cvePollIntervalMinutes.value = Number(settings.value.cve_poll_interval) || 60
      }
      if (settings.value.parallel_workers) {
        parallelWorkers.value = Number(settings.value.parallel_workers) || 5
      }
      if (settings.value.inter_request_delay_ms) {
        requestDelayMs.value = Number(settings.value.inter_request_delay_ms) || 200
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function saveSettingsAction(newSettings: Record<string, string>) {
    error.value = null
    try {
      await saveSettings(newSettings)
      settings.value = { ...settings.value, ...newSettings }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function loadAuditLog(limit?: number, actionType?: string) {
    error.value = null
    try {
      auditLog.value = await listAuditLog(limit, actionType)
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    scanIntervalMinutes,
    cvePollIntervalMinutes,
    parallelWorkers,
    requestDelayMs,
    darkMode,
    rateLimitGithub,
    rateLimitGitlab,
    settings,
    auditLog,
    isLoading,
    error,
    refreshRateLimit,
    loadSettings,
    saveSettingsAction,
    loadAuditLog,
  }
})
