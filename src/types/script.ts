export interface ScriptPreset {
  id: string
  name: string
  command: string
  description: string
}

export interface ScriptRepoResult {
  repoId: string
  exitCode: number
  stdout: string
  stderr: string
  durationMs: number
}

export interface ScriptRun {
  id: string
  command: string
  repoIds: string[]
  results: ScriptRepoResult[]
  status: 'running' | 'completed' | 'aborted'
  startedAt: string
  completedAt: string | null
}
