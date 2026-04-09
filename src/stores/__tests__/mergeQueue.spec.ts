import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useMergeQueueStore } from '@/stores/mergeQueue'
import * as mergeQueueService from '@/services/mergeQueue'
import type { FlotillaPr } from '@/types/mergeQueue'

vi.mock('@/services/mergeQueue')

const makePr = (overrides: Partial<FlotillaPr> = {}): FlotillaPr => ({
  repoId: 'github:org/repo',
  prNumber: 1,
  title: 'fix: patch dep',
  state: 'open',
  mergeable: 'MERGEABLE',
  ciStatus: 'success',
  operationId: 'op-1',
  createdAt: '2026-04-01T00:00:00Z',
  htmlUrl: 'https://github.com/org/repo/pull/1',
  ...overrides,
})

describe('mergeQueue store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initialises with empty state', () => {
    const store = useMergeQueueStore()
    expect(store.prs).toHaveLength(0)
    expect(store.isLoading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.greenPrs).toHaveLength(0)
    expect(store.prCount).toBe(0)
  })

  it('loads PRs', async () => {
    vi.mocked(mergeQueueService.listFlotillaPrs).mockResolvedValue([
      makePr(),
      makePr({ prNumber: 2, ciStatus: 'failure' }),
    ])
    const store = useMergeQueueStore()
    await store.loadPrs()
    expect(store.prs).toHaveLength(2)
    expect(store.prCount).toBe(2)
    expect(store.isLoading).toBe(false)
  })

  it('greenPrs computed filters correctly', async () => {
    vi.mocked(mergeQueueService.listFlotillaPrs).mockResolvedValue([
      makePr({ prNumber: 1, ciStatus: 'success', mergeable: 'MERGEABLE' }),
      makePr({ prNumber: 2, ciStatus: 'failure', mergeable: 'MERGEABLE' }),
      makePr({ prNumber: 3, ciStatus: 'success', mergeable: 'CONFLICTING' }),
      makePr({ prNumber: 4, ciStatus: 'success', mergeable: 'MERGEABLE' }),
    ])
    const store = useMergeQueueStore()
    await store.loadPrs()
    expect(store.greenPrs).toHaveLength(2)
    expect(store.greenPrs.map(p => p.prNumber)).toEqual([1, 4])
  })
})
