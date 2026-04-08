export type CveSeverity = 'critical' | 'high' | 'medium' | 'low'
export type CveStatus = 'new' | 'acknowledged' | 'patched' | 'dismissed'

export interface CveAlert {
  id: string
  packageName: string
  ecosystem: string
  severity: CveSeverity
  summary: string
  affectedVersionRange: string
  fixedVersion: string | null
  publishedAt: string
  detectedAt: string
  affectedRepos: string[]
  status: CveStatus
}

export interface IncidentEvent {
  timestamp: string
  eventType: string
  repoId: string | null
  detail: string
}

export interface IncidentTimeline {
  cveId: string
  publishedAt: string
  detectedAt: string
  events: IncidentEvent[]
}

export interface BlastRadius {
  cveId: string
  directRepos: string[]
  transitiveRepos: string[]
  dependencyPaths: string[][]
}
