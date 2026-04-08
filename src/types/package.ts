export type Ecosystem = 'npm' | 'composer' | 'pip' | 'cargo' | 'go'

export interface RepoPackage {
  repoId: string
  ecosystem: Ecosystem
  name: string
  version: string
  isDev: boolean
  scannedAt: string
}

export interface ChangelogEntry {
  version: string
  body: string
  publishedAt: string
  isBreaking: boolean
}

export interface PackageRow {
  name: string
  ecosystem: Ecosystem
  versionsByRepo: Record<string, string>
  latestVersion: string | null
}

export interface DependencyMatrix {
  packages: PackageRow[]
  repoIds: string[]
}
