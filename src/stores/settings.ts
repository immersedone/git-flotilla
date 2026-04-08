import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface RateLimitInfo {
  remaining: number
  limit: number
  resetAt: string
}

export const useSettingsStore = defineStore('settings', () => {
  const scanIntervalMinutes = ref<number | null>(1440)  // daily
  const cvePollIntervalMinutes = ref<number | null>(60) // hourly
  const parallelWorkers = ref(5)
  const requestDelayMs = ref(200)
  const darkMode = ref(true)
  const rateLimitGithub = ref<RateLimitInfo | null>(null)
  const rateLimitGitlab = ref<RateLimitInfo | null>(null)

  return {
    scanIntervalMinutes,
    cvePollIntervalMinutes,
    parallelWorkers,
    requestDelayMs,
    darkMode,
    rateLimitGithub,
    rateLimitGitlab,
  }
})
