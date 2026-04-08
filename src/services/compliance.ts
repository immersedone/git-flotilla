import { invoke } from '@tauri-apps/api/core'
import type { SecretFinding, LicenceFinding, BranchProtectionStatus } from '@/types/compliance'

export function scanSecrets(repoIds: string[]): Promise<SecretFinding[]> {
  return invoke('scan_secrets', { repoIds })
}

export function scanLicences(repoIds: string[], blockedLicences: string[]): Promise<LicenceFinding[]> {
  return invoke('scan_licences', { repoIds, blockedLicences })
}

export function auditBranchProtection(repoIds: string[]): Promise<BranchProtectionStatus[]> {
  return invoke('audit_branch_protection', { repoIds })
}

export function archiveRepos(repoIds: string[]): Promise<string[]> {
  return invoke('archive_repos', { repoIds })
}
