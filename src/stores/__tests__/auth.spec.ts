import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useAuthStore } from '@/stores/auth'
import * as authService from '@/services/auth'

vi.mock('@/services/auth')

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('starts with no accounts', () => {
    const store = useAuthStore()
    expect(store.accounts).toHaveLength(0)
    expect(store.hasAccounts).toBe(false)
  })

  it('loadAccounts populates accounts', async () => {
    vi.mocked(authService.listAccounts).mockResolvedValue([
      { id: 'github:octocat', provider: 'github' as const, username: 'octocat', scopes: ['repo'], missingScopes: [], avatarUrl: null },
    ])
    const store = useAuthStore()
    await store.loadAccounts()
    expect(store.accounts).toHaveLength(1)
    expect(store.hasAccounts).toBe(true)
    expect(store.githubAccount?.username).toBe('octocat')
  })

  it('addAccountAction calls addAccount service and appends to accounts', async () => {
    const newAccount = { id: 'github:octocat', provider: 'github' as const, username: 'octocat', scopes: ['repo', 'workflow'], missingScopes: [], avatarUrl: null }
    vi.mocked(authService.addAccount).mockResolvedValue(newAccount)
    const store = useAuthStore()
    await store.addAccountAction('github', 'ghp_test123')
    expect(authService.addAccount).toHaveBeenCalledWith('github', 'ghp_test123')
    expect(store.accounts).toHaveLength(1)
  })

  it('removeAccountAction calls removeAccount and removes from list', async () => {
    vi.mocked(authService.listAccounts).mockResolvedValue([
      { id: 'github:octocat', provider: 'github' as const, username: 'octocat', scopes: [], missingScopes: [], avatarUrl: null },
    ])
    vi.mocked(authService.removeAccount).mockResolvedValue(undefined)
    const store = useAuthStore()
    await store.loadAccounts()
    expect(store.accounts).toHaveLength(1)
    await store.removeAccountAction('github:octocat')
    expect(store.accounts).toHaveLength(0)
  })

  it('sets isLoading true while loading', async () => {
    let resolve!: () => void
    vi.mocked(authService.listAccounts).mockReturnValue(new Promise(r => { resolve = () => r([]) }))
    const store = useAuthStore()
    const p = store.loadAccounts()
    expect(store.isLoading).toBe(true)
    resolve()
    await p
    expect(store.isLoading).toBe(false)
  })
})
