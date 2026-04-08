export interface FlotillaPr {
  repoId: string
  prNumber: number
  title: string
  state: string
  mergeable: string | null
  ciStatus: string | null
  operationId: string
  createdAt: string
  htmlUrl: string
}
