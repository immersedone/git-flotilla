import { invoke } from '@tauri-apps/api/core'
import type { DependencyMatrix, ChangelogEntry } from '@/types/package'

export function getDependencyMatrix(
  repoListId?: string,
  ecosystem?: string,
): Promise<DependencyMatrix> {
  return invoke('get_dependency_matrix', {
    repoListId: repoListId ?? null,
    ecosystem: ecosystem ?? null,
  })
}

export function getPackageChangelog(
  packageName: string,
  ecosystem: string,
  fromVersion: string,
  toVersion: string,
): Promise<ChangelogEntry[]> {
  return invoke('get_package_changelog', { packageName, ecosystem, fromVersion, toVersion })
}

export function exportMatrixCsv(repoListId?: string): Promise<string> {
  return invoke('export_matrix_csv', { repoListId: repoListId ?? null })
}
