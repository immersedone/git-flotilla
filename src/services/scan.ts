import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ScanResult, ScanProgressEvent } from '@/types/scan'

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

export function onScanProgress(
  callback: (event: ScanProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<ScanProgressEvent>('scan-progress', (event) => {
    callback(event.payload)
  })
}
