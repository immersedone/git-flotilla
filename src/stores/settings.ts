import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getRateLimitStatus } from '@/services/settings'

export interface RateLimitInfo {
  remaining: number
  limit: number
  resetEpoch: number
}

export const useSettingsStore = defineStore('settings', () => {
  const scanIntervalMinutes    = ref<number | null>(1440)
  const cvePollIntervalMinutes = ref<number | null>(60)
  const parallelWorkers        = ref(5)
  const requestDelayMs         = ref(200)
  const darkMode               = ref(true)
  const rateLimitGithub        = ref<RateLimitInfo | null>(null)
  const rateLimitGitlab        = ref<RateLimitInfo | null>(null)

  async function refreshRateLimit() {
    try {
      const status = await getRateLimitStatus()
      rateLimitGithub.value = status.github
      rateLimitGitlab.value = status.gitlab
    } catch {
      // non-fatal — rate limit display is informational
    }
  }

  return {
    scanIntervalMinutes, cvePollIntervalMinutes,
    parallelWorkers, requestDelayMs, darkMode,
    rateLimitGithub, rateLimitGitlab,
    refreshRateLimit,
  }
})
