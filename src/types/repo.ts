export interface Repo {
  id: string              // "{provider}:{owner}/{name}"
  provider: 'github' | 'gitlab'
  owner: string
  name: string
  fullName: string
  url: string
  defaultBranch: string
  isPrivate: boolean
  lastScannedAt: string | null
  tags: string[]
}

export interface RepoList {
  id: string
  name: string
  description: string
  repoIds: string[]
  parentId: string | null
  excludePatterns: string[]
  createdAt: string
  updatedAt: string
}
