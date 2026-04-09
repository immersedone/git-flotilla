import { invoke } from '@tauri-apps/api/core'
import type { RateLimitInfo } from '@/types/settings'

export interface RateLimitStatus {
  github: RateLimitInfo | null
  gitlab: RateLimitInfo | null
}

export function getRateLimitStatus(): Promise<RateLimitStatus> {
  return invoke('get_rate_limit_status')
}
