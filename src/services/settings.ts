import { invoke } from '@tauri-apps/api/core'
import type { RateLimitInfo } from '@/types/settings'

export interface RateLimitStatus {
  github: RateLimitInfo | null
  gitlab: RateLimitInfo | null
}

export function getRateLimitStatus(): Promise<RateLimitStatus> {
  return invoke('get_rate_limit_status')
}

export function getSettings(): Promise<Record<string, string>> {
  return invoke('get_settings')
}

export function saveSettings(settings: Record<string, string>): Promise<void> {
  return invoke('save_settings', { settings })
}

export function listAuditLog(
  limit?: number,
  actionType?: string,
): Promise<Array<Record<string, unknown>>> {
  return invoke('list_audit_log', {
    limit: limit ?? null,
    actionType: actionType ?? null,
  })
}
