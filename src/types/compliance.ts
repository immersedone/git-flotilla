export interface SecretFinding {
  repoId: string
  filePath: string
  lineNumber: number
  secretType: string
  matchedPattern: string
}

export interface LicenceFinding {
  repoId: string
  packageName: string
  ecosystem: string
  licence: string
  isFlagged: boolean
  flagReason: string | null
}

export interface BranchProtectionStatus {
  repoId: string
  branch: string
  requiresReviews: boolean
  requiresStatusChecks: boolean
  restrictsPushes: boolean
  isCompliant: boolean
  issues: string[]
}
