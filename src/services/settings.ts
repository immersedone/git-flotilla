import { invoke } from '@tauri-apps/api/core'
import type { RateLimitInfo } from '@/types/settings'

export interface RateLimitStatus {
  github: RateLimitInfo | null
  gitlab: RateLimitInfo | null
}

export interface AppNotification {
  id: string
  notificationType: string
  title: string
  message: string
  timestamp: string
  isRead: boolean
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

export function listNotifications(): Promise<AppNotification[]> {
  return invoke('list_notifications')
}

export function markNotificationRead(id: string): Promise<void> {
  return invoke('mark_notification_read', { id })
}

export function clearNotifications(): Promise<void> {
  return invoke('clear_notifications')
}

export function exportAuditLogCsv(): Promise<string> {
  return invoke('export_audit_log_csv')
}

export function exportHealthReportCsv(repoListId?: string): Promise<string> {
  return invoke('export_health_report_csv', {
    repoListId: repoListId ?? null,
  })
}

export function exportCveReportCsv(): Promise<string> {
  return invoke('export_cve_report_csv')
}

export function exportConfig(): Promise<string> {
  return invoke('export_config')
}

export function importConfig(yaml: string): Promise<void> {
  return invoke('import_config', { yaml })
}
