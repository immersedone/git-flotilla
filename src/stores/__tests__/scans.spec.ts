import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useScansStore } from '@/stores/scans'
import * as scanService from '@/services/scan'
import type { ScanResult } from '@/types/scan'

vi.mock('@/services/scan', () => ({
  scanRepo: vi.fn(),
  scanRepoList: vi.fn(),
  getScanResult: vi.fn(),
  listScanResults: vi.fn(),
  abortScan: vi.fn(),
  onScanProgress: vi.fn(),
}))

const makeScanResult = (repoId: string, healthScore: number): ScanResult => ({
  repoId,
  scannedAt: '2026-04-09T00:00:00Z',
  manifestPaths: ['package.json'],
  nodeVersion: '20.11.0',
  nodeVersionSource: '.nvmrc',
  phpVersion: null,
  packageManager: 'pnpm',
  packageManagerVersion: '9.0.0',
  hasDevelop: false,
  lastPushed: '2026-04-08T12:00:00Z',
  hasDotEnvExample: true,
  workflowFiles: ['.github/workflows/ci.yml'],
  healthScore,
  flags: [],
  excluded: false,
  excludeReason: null,
})

describe('scans store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initialises with empty state', () => {
    const store = useScansStore()
    expect(store.results).toHaveLength(0)
    expect(store.isScanning).toBe(false)
    expect(store.scanProgress).toEqual({ current: 0, total: 0 })
    expect(store.error).toBeNull()
  })

  it('loads scan results', async () => {
    const mockResults = [makeScanResult('github:org/repo-a', 85)]
    vi.mocked(scanService.listScanResults).mockResolvedValue(mockResults)

    const store = useScansStore()
    await store.loadResults()

    expect(scanService.listScanResults).toHaveBeenCalledWith(undefined)
    expect(store.results).toHaveLength(1)
    expect(store.results[0].repoId).toBe('github:org/repo-a')
  })

  it('handles loadResults error', async () => {
    vi.mocked(scanService.listScanResults).mockRejectedValue(new Error('Network error'))

    const store = useScansStore()
    await expect(store.loadResults()).rejects.toThrow('Network error')
    expect(store.error).toBe('Error: Network error')
  })

  it('computes averageHealthScore', () => {
    const store = useScansStore()
    store.results = [
      makeScanResult('github:org/repo-a', 80),
      makeScanResult('github:org/repo-b', 60),
    ]
    expect(store.averageHealthScore).toBe(70)
  })

  it('averageHealthScore is 0 when no results', () => {
    const store = useScansStore()
    expect(store.averageHealthScore).toBe(0)
  })
})
