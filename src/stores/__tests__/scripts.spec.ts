import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useScriptsStore } from '@/stores/scripts'
import * as scriptsService from '@/services/scripts'

vi.mock('@/services/scripts')

describe('scripts store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initialises with empty state', () => {
    const store = useScriptsStore()
    expect(store.presets).toHaveLength(0)
    expect(store.activeRun).toBeNull()
    expect(store.isRunning).toBe(false)
    expect(store.isLoading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('loads presets', async () => {
    vi.mocked(scriptsService.listPresets).mockResolvedValue([
      { id: 'p1', name: 'Outdated check', command: 'npm outdated', description: 'Check outdated deps' },
    ])
    const store = useScriptsStore()
    await store.loadPresets()
    expect(store.presets).toHaveLength(1)
    expect(store.presets[0].name).toBe('Outdated check')
    expect(store.isLoading).toBe(false)
  })

  it('handles error on loadPresets', async () => {
    vi.mocked(scriptsService.listPresets).mockRejectedValue(new Error('Network error'))
    const store = useScriptsStore()
    await store.loadPresets()
    expect(store.error).toBe('Error: Network error')
    expect(store.presets).toHaveLength(0)
    expect(store.isLoading).toBe(false)
  })

  it('saves a preset', async () => {
    const preset = { id: 'p2', name: 'Lint', command: 'npm run lint', description: '' }
    vi.mocked(scriptsService.savePreset).mockResolvedValue(preset)
    const store = useScriptsStore()
    const result = await store.savePresetAction(preset)
    expect(result).toEqual(preset)
    expect(store.presets).toHaveLength(1)
  })

  it('deletes a preset', async () => {
    vi.mocked(scriptsService.listPresets).mockResolvedValue([
      { id: 'p1', name: 'Test', command: 'npm test', description: '' },
    ])
    vi.mocked(scriptsService.deletePreset).mockResolvedValue()
    const store = useScriptsStore()
    await store.loadPresets()
    expect(store.presets).toHaveLength(1)
    await store.deletePresetAction('p1')
    expect(store.presets).toHaveLength(0)
  })

  it('runs a script and stores the result', async () => {
    const mockRun = {
      id: 'run-1',
      command: 'echo hello',
      repoIds: ['github:org/repo'],
      results: [{ repoId: 'github:org/repo', exitCode: 0, stdout: 'hello\n', stderr: '', durationMs: 42 }],
      status: 'completed' as const,
      startedAt: '2026-04-09T00:00:00Z',
      completedAt: '2026-04-09T00:00:01Z',
    }
    vi.mocked(scriptsService.runScript).mockResolvedValue('run-1')
    vi.mocked(scriptsService.getScriptRun).mockResolvedValue(mockRun)
    const store = useScriptsStore()
    await store.runScriptAction('echo hello', ['github:org/repo'])
    expect(store.activeRun).toEqual(mockRun)
    expect(store.isRunning).toBe(false)
  })
})
