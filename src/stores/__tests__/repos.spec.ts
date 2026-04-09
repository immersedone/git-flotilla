import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useReposStore } from '@/stores/repos'
import * as reposService from '@/services/repos'

vi.mock('@/services/repos')

describe('repos store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('starts empty', () => {
    const store = useReposStore()
    expect(store.repos).toHaveLength(0)
  })

  it('loadRepos fetches and stores repos', async () => {
    vi.mocked(reposService.listRepos).mockResolvedValue([
      { id: 'github:org/repo-a', provider: 'github', owner: 'org', name: 'repo-a',
        fullName: 'org/repo-a', url: 'https://github.com/org/repo-a',
        defaultBranch: 'main', isPrivate: false, lastScannedAt: null, tags: [] },
    ])
    const store = useReposStore()
    await store.loadRepos()
    expect(store.repos).toHaveLength(1)
    expect(store.repos[0].fullName).toBe('org/repo-a')
  })

  it('filteredRepos filters by search query', async () => {
    vi.mocked(reposService.listRepos).mockResolvedValue([
      { id: 'github:org/frontend', provider: 'github', owner: 'org', name: 'frontend',
        fullName: 'org/frontend', url: '', defaultBranch: 'main', isPrivate: false, lastScannedAt: null, tags: ['vue'] },
      { id: 'github:org/backend', provider: 'github', owner: 'org', name: 'backend',
        fullName: 'org/backend', url: '', defaultBranch: 'main', isPrivate: false, lastScannedAt: null, tags: [] },
    ])
    const store = useReposStore()
    await store.loadRepos()
    store.searchQuery = 'front'
    expect(store.filteredRepos).toHaveLength(1)
    expect(store.filteredRepos[0].name).toBe('frontend')
  })
})
