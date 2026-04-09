import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { usePackagesStore } from '@/stores/packages'
import * as packagesService from '@/services/packages'
import type { DependencyMatrix } from '@/types/package'

vi.mock('@/services/packages', () => ({
  getDependencyMatrix: vi.fn(),
  getPackageChangelog: vi.fn(),
  exportMatrixCsv: vi.fn(),
}))

const makeMatrix = (): DependencyMatrix => ({
  packages: [
    {
      name: 'vue',
      ecosystem: 'npm',
      versionsByRepo: { 'github:org/repo-a': '3.4.0', 'github:org/repo-b': '3.5.0' },
      latestVersion: '3.5.0',
      repoCount: 2,
      hasDrift: true,
      isDevOnly: false,
    },
    {
      name: 'laravel/framework',
      ecosystem: 'composer',
      versionsByRepo: { 'github:org/repo-c': '11.0.0' },
      latestVersion: '11.0.0',
      repoCount: 1,
      hasDrift: false,
      isDevOnly: false,
    },
    {
      name: 'vitest',
      ecosystem: 'npm',
      versionsByRepo: { 'github:org/repo-a': '4.1.0' },
      latestVersion: '4.1.0',
      repoCount: 1,
      hasDrift: false,
      isDevOnly: true,
    },
  ],
  repoIds: ['github:org/repo-a', 'github:org/repo-b', 'github:org/repo-c'],
})

describe('packages store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initialises with empty state', () => {
    const store = usePackagesStore()
    expect(store.matrix).toBeNull()
    expect(store.isLoading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.selectedEcosystem).toBeNull()
    expect(store.selectedRepoListId).toBeNull()
    expect(store.changelog).toHaveLength(0)
    expect(store.changelogLoading).toBe(false)
    expect(store.searchQuery).toBe('')
    expect(store.filteredPackages).toHaveLength(0)
    expect(store.ecosystems).toHaveLength(0)
    expect(store.driftCount).toBe(0)
  })

  it('loads matrix', async () => {
    const mockMatrix = makeMatrix()
    vi.mocked(packagesService.getDependencyMatrix).mockResolvedValue(mockMatrix)

    const store = usePackagesStore()
    await store.loadMatrix('list-1', 'npm')

    expect(packagesService.getDependencyMatrix).toHaveBeenCalledWith('list-1', 'npm')
    expect(store.matrix).toEqual(mockMatrix)
    expect(store.isLoading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('handles loadMatrix error', async () => {
    vi.mocked(packagesService.getDependencyMatrix).mockRejectedValue(new Error('API failure'))

    const store = usePackagesStore()
    await expect(store.loadMatrix()).rejects.toThrow('API failure')
    expect(store.error).toBe('Error: API failure')
    expect(store.matrix).toBeNull()
    expect(store.isLoading).toBe(false)
  })

  it('filteredPackages filters by ecosystem', () => {
    const store = usePackagesStore()
    store.matrix = makeMatrix()
    store.selectedEcosystem = 'npm'

    expect(store.filteredPackages).toHaveLength(2)
    expect(store.filteredPackages.every(p => p.ecosystem === 'npm')).toBe(true)
  })

  it('filteredPackages filters by searchQuery', () => {
    const store = usePackagesStore()
    store.matrix = makeMatrix()
    store.searchQuery = 'vue'

    expect(store.filteredPackages).toHaveLength(1)
    expect(store.filteredPackages[0].name).toBe('vue')
  })

  it('filteredPackages filters by both ecosystem and searchQuery', () => {
    const store = usePackagesStore()
    store.matrix = makeMatrix()
    store.selectedEcosystem = 'npm'
    store.searchQuery = 'vitest'

    expect(store.filteredPackages).toHaveLength(1)
    expect(store.filteredPackages[0].name).toBe('vitest')
  })

  it('computes ecosystems as sorted unique list', () => {
    const store = usePackagesStore()
    store.matrix = makeMatrix()

    expect(store.ecosystems).toEqual(['composer', 'npm'])
  })

  it('computes driftCount', () => {
    const store = usePackagesStore()
    store.matrix = makeMatrix()

    expect(store.driftCount).toBe(1)
  })

  it('fetches changelog', async () => {
    const mockEntries = [
      { version: '3.5.0', body: 'New features', publishedAt: '2026-04-01T00:00:00Z', isBreaking: false },
    ]
    vi.mocked(packagesService.getPackageChangelog).mockResolvedValue(mockEntries)

    const store = usePackagesStore()
    await store.fetchChangelog('vue', 'npm', '3.4.0', '3.5.0')

    expect(packagesService.getPackageChangelog).toHaveBeenCalledWith('vue', 'npm', '3.4.0', '3.5.0')
    expect(store.changelog).toEqual(mockEntries)
    expect(store.changelogLoading).toBe(false)
  })

  it('exports CSV', async () => {
    vi.mocked(packagesService.exportMatrixCsv).mockResolvedValue('name,ecosystem\nvue,npm')

    const store = usePackagesStore()
    const csv = await store.exportCsv('list-1')

    expect(packagesService.exportMatrixCsv).toHaveBeenCalledWith('list-1')
    expect(csv).toBe('name,ecosystem\nvue,npm')
  })
})
