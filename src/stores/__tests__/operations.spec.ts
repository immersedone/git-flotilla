import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useOperationsStore } from '@/stores/operations'
import type { BatchOperation, OperationStatus } from '@/types/operation'

vi.mock('@/services/operations', () => ({
  listOperations: vi.fn(),
  createOperation: vi.fn(),
  runOperation: vi.fn(),
  abortOperation: vi.fn(),
  getOperation: vi.fn(),
  rollbackOperation: vi.fn(),
  onOperationProgress: vi.fn().mockResolvedValue(vi.fn()),
}))

import {
  listOperations,
  createOperation,
} from '@/services/operations'

const mockedListOperations = vi.mocked(listOperations)
const mockedCreateOperation = vi.mocked(createOperation)

function makeOp(overrides: Partial<BatchOperation> = {}): BatchOperation {
  return {
    id: 'op-1',
    type: 'package_pin',
    mode: 'pin',
    status: 'pending',
    targetRepoIds: ['repo-1'],
    completedRepoIds: [],
    versionMap: null,
    createdAt: '2026-04-09T00:00:00Z',
    completedAt: null,
    results: [],
    isDryRun: true,
    skipCi: false,
    ...overrides,
  }
}

describe('useOperationsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initialises with empty state', () => {
    const store = useOperationsStore()

    expect(store.operations).toEqual([])
    expect(store.activeOperation).toBeNull()
    expect(store.isLoading).toBe(false)
    expect(store.isRunning).toBe(false)
    expect(store.error).toBeNull()
    expect(store.progress).toEqual({ current: 0, total: 0 })
    expect(store.repoStatuses).toEqual({})
  })

  it('loads operations', async () => {
    const ops = [makeOp(), makeOp({ id: 'op-2', status: 'completed' })]
    mockedListOperations.mockResolvedValueOnce(ops)

    const store = useOperationsStore()
    await store.loadOperations()

    expect(mockedListOperations).toHaveBeenCalledOnce()
    expect(store.operations).toEqual(ops)
    expect(store.isLoading).toBe(false)
  })

  it('handles loadOperations error', async () => {
    mockedListOperations.mockRejectedValueOnce(new Error('Network error'))

    const store = useOperationsStore()
    await store.loadOperations()

    expect(store.error).toBe('Error: Network error')
    expect(store.operations).toEqual([])
    expect(store.isLoading).toBe(false)
  })

  it('creates operation', async () => {
    const newOp = makeOp({ id: 'op-new' })
    mockedCreateOperation.mockResolvedValueOnce(newOp)

    const store = useOperationsStore()
    const result = await store.createOp({
      operationType: 'package_pin',
      mode: 'pin',
      targetRepoIds: ['repo-1'],
      packageName: 'lodash',
      targetVersion: '4.17.21',
      isDryRun: true,
      skipCi: false,
      alsoTargetBranches: [],
      divergenceCheck: false,
    })

    expect(mockedCreateOperation).toHaveBeenCalledOnce()
    expect(result).toEqual(newOp)
    expect(store.operations).toHaveLength(1)
    expect(store.operations[0].id).toBe('op-new')
  })

  it('computed pendingOps filters correctly', () => {
    const store = useOperationsStore()

    const statuses: OperationStatus[] = [
      'pending',
      'running',
      'completed',
      'pending',
      'failed',
    ]
    store.operations = statuses.map((status, i) =>
      makeOp({ id: `op-${i}`, status }),
    )

    expect(store.pendingOps).toHaveLength(2)
    expect(store.pendingOps.map((op) => op.id)).toEqual(['op-0', 'op-3'])
    expect(store.runningOps).toHaveLength(1)
    expect(store.completedOps).toHaveLength(1)
  })
})
