export interface ScanFlag {
  flagType: string
  message: string
  severity: 'critical' | 'high' | 'medium' | 'low' | 'info'
}

export interface ScanResult {
  repoId: string
  scannedAt: string
  manifestPaths: string[]
  nodeVersion: string | null
  nodeVersionSource: string | null
  phpVersion: string | null
  packageManager: 'npm' | 'pnpm' | 'yarn' | 'bun' | 'composer' | null
  packageManagerVersion: string | null
  hasDevelop: boolean
  lastPushed: string | null
  hasDotEnvExample: boolean
  workflowFiles: string[]
  healthScore: number
  flags: ScanFlag[]
  excluded: boolean
  excludeReason: string | null
}

export interface ScanProgressEvent {
  repoId: string
  status: 'scanning' | 'done' | 'failed'
  current: number
  total: number
  error: string | null
}

export interface BatchScanSummary {
  total: number
  succeeded: number
  failed: number
  inProgress: number
}
