import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCveStore } from '@/stores/cve'
import type { CveAlert } from '@/types/cve'

vi.mock('@/services/cve', () => ({
  listCveAlerts: vi.fn(),
  checkCves: vi.fn(),
  acknowledgeCve: vi.fn(),
  dismissCve: vi.fn(),
  snoozeCve: vi.fn(),
  getBlastRadius: vi.fn(),
  listWatchlist: vi.fn(),
  addToWatchlist: vi.fn(),
  removeFromWatchlist: vi.fn(),
}))

import {
  listCveAlerts,
  checkCves,
} from '@/services/cve'

const mockedListCveAlerts = vi.mocked(listCveAlerts)
const mockedCheckCves = vi.mocked(checkCves)

function makeAlert(overrides: Partial<CveAlert> = {}): CveAlert {
  return {
    id: 'CVE-2024-00001',
    packageName: 'lodash',
    ecosystem: 'npm',
    severity: 'high',
    summary: 'Prototype pollution',
    affectedVersionRange: '<4.17.21',
    fixedVersion: '4.17.21',
    publishedAt: '2024-01-01T00:00:00Z',
    detectedAt: '2024-01-02T00:00:00Z',
    affectedRepos: ['github:org/repo1'],
    status: 'new',
    ...overrides,
  }
}

describe('useCveStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initialises with empty state', () => {
    const store = useCveStore()
    expect(store.alerts).toEqual([])
    expect(store.isLoading).toBe(false)
    expect(store.lastCheckedAt).toBeNull()
    expect(store.error).toBeNull()
    expect(store.watchlist).toEqual([])
    expect(store.selectedSeverity).toBeNull()
    expect(store.selectedStatus).toBeNull()
    expect(store.blastRadius).toBeNull()
    expect(store.badgeCount).toBe(0)
  })

  it('loads alerts', async () => {
    const alerts = [makeAlert(), makeAlert({ id: 'CVE-2024-00002', severity: 'critical' })]
    mockedListCveAlerts.mockResolvedValue(alerts)

    const store = useCveStore()
    await store.loadAlerts()

    expect(store.alerts).toEqual(alerts)
    expect(store.isLoading).toBe(false)
    expect(store.error).toBeNull()
    expect(mockedListCveAlerts).toHaveBeenCalledWith(undefined, undefined)
  })

  it('handles loadAlerts error', async () => {
    mockedListCveAlerts.mockRejectedValue(new Error('Network error'))

    const store = useCveStore()
    await expect(store.loadAlerts()).rejects.toThrow('Network error')

    expect(store.error).toBe('Error: Network error')
    expect(store.isLoading).toBe(false)
    expect(store.alerts).toEqual([])
  })

  it('filteredAlerts filters by severity', () => {
    const store = useCveStore()
    store.alerts = [
      makeAlert({ id: 'CVE-1', severity: 'critical' }),
      makeAlert({ id: 'CVE-2', severity: 'high' }),
      makeAlert({ id: 'CVE-3', severity: 'low' }),
    ]

    expect(store.filteredAlerts).toHaveLength(3)

    store.selectedSeverity = 'critical'
    expect(store.filteredAlerts).toHaveLength(1)
    expect(store.filteredAlerts[0].id).toBe('CVE-1')

    store.selectedSeverity = null
    store.selectedStatus = 'new'
    expect(store.filteredAlerts).toHaveLength(3)
  })

  it('badgeCount counts new critical + high', () => {
    const store = useCveStore()
    store.alerts = [
      makeAlert({ id: 'CVE-1', severity: 'critical', status: 'new' }),
      makeAlert({ id: 'CVE-2', severity: 'high', status: 'new' }),
      makeAlert({ id: 'CVE-3', severity: 'high', status: 'acknowledged' }),
      makeAlert({ id: 'CVE-4', severity: 'medium', status: 'new' }),
      makeAlert({ id: 'CVE-5', severity: 'critical', status: 'dismissed' }),
    ]

    // Only new critical (CVE-1) + new high (CVE-2) = 2
    expect(store.badgeCount).toBe(2)
    expect(store.criticalCount).toBe(1)
    expect(store.highCount).toBe(1)
  })

  it('runCheck updates alerts and lastCheckedAt', async () => {
    const alerts = [makeAlert()]
    mockedCheckCves.mockResolvedValue(alerts)

    const store = useCveStore()
    await store.runCheck()

    expect(store.alerts).toEqual(alerts)
    expect(store.lastCheckedAt).not.toBeNull()
    expect(store.isLoading).toBe(false)
  })
})
