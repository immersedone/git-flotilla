export type OperationType =
  | 'file_update'
  | 'package_pin'
  | 'package_bump'
  | 'workflow_sync'
  | 'script_run'
  | 'pr_create'
  | 'commit'

export type OperationMode = 'pin' | 'bump' | null

export type OperationStatus =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'rolled_back'
  | 'paused'

export interface OperationResult {
  repoId: string
  status: string
  prUrl: string | null
  error: string | null
  diff: string | null
}

export interface BatchOperation {
  id: string
  type: OperationType
  mode: OperationMode
  status: OperationStatus
  targetRepoIds: string[]
  completedRepoIds: string[]
  versionMap: Record<string, string> | null
  createdAt: string
  completedAt: string | null
  results: OperationResult[]
  isDryRun: boolean
  skipCi: boolean
}

export interface ValidateResult {
  repoId: string
  isApplied: boolean
  currentVersion: string | null
  hasOverrides: boolean
}

export interface CreateOperationInput {
  operationType: OperationType
  mode?: OperationMode
  targetRepoIds: string[]
  packageName?: string
  targetVersion?: string
  versionMap?: Record<string, string>
  filePath?: string
  fileContent?: string
  prTitleTemplate?: string
  prBodyTemplate?: string
  branchPrefix?: string
  label?: string
  isDryRun: boolean
  skipCi: boolean
  alsoTargetBranches: string[]
  divergenceCheck: boolean
  divergenceThreshold?: number
}
