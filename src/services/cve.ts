import { invoke } from './tauri'
import type { CveAlert, IncidentTimeline, BlastRadius } from '@/types/cve'

export function checkCves(): Promise<CveAlert[]> {
  return invoke('check_cves')
}

export function listCveAlerts(severity?: string, status?: string): Promise<CveAlert[]> {
  return invoke('list_cve_alerts', {
    severity: severity ?? null,
    status: status ?? null,
  })
}

export function acknowledgeCve(cveId: string, repoId?: string): Promise<void> {
  return invoke('acknowledge_cve', { cveId, repoId: repoId ?? null })
}

export function dismissCve(cveId: string, repoId?: string): Promise<void> {
  return invoke('dismiss_cve', { cveId, repoId: repoId ?? null })
}

export function snoozeCve(cveId: string, repoId: string | undefined, days: number): Promise<void> {
  return invoke('snooze_cve', { cveId, repoId: repoId ?? null, days })
}

export function getCveIncident(cveId: string): Promise<IncidentTimeline> {
  return invoke('get_cve_incident', { cveId })
}

export function getBlastRadius(cveId: string): Promise<BlastRadius> {
  return invoke('get_blast_radius', { cveId })
}

export function addToWatchlist(packageName: string, ecosystem: string): Promise<void> {
  return invoke('add_to_watchlist', { packageName, ecosystem })
}

export function removeFromWatchlist(packageName: string, ecosystem: string): Promise<void> {
  return invoke('remove_from_watchlist', { packageName, ecosystem })
}

export function listWatchlist(): Promise<Array<{ packageName: string; ecosystem: string; addedAt: string }>> {
  return invoke('list_watchlist')
}
