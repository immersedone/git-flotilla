import { invoke } from '@tauri-apps/api/core'
import type { ScanResult } from '@/types/scan'

export function scanRepo(repoId: string): Promise<ScanResult> {
  return invoke('scan_repo', { repoId })
}

export function scanRepoList(listId: string): Promise<string> {
  return invoke('scan_repo_list', { listId })
}

export function getScanResult(repoId: string): Promise<ScanResult> {
  return invoke('get_scan_result', { repoId })
}

export function listScanResults(repoListId?: string): Promise<ScanResult[]> {
  return invoke('list_scan_results', { repoListId: repoListId ?? null })
}

export function abortScan(operationId: string): Promise<void> {
  return invoke('abort_scan', { operationId })
}
