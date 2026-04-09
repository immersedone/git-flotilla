import { invoke } from './tauri'
import type { FlotillaPr } from '@/types/mergeQueue'

export function listFlotillaPrs(operationId?: string): Promise<FlotillaPr[]> {
  return invoke('list_flotilla_prs', { operationId: operationId ?? null })
}

export function mergePr(repoId: string, prNumber: number): Promise<void> {
  return invoke('merge_pr', { repoId, prNumber })
}

export function mergeAllGreen(operationId?: string): Promise<number> {
  return invoke('merge_all_green', { operationId: operationId ?? null })
}
