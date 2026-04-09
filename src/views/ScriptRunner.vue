<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useScriptsStore } from '@/stores/scripts'
import { useReposStore } from '@/stores/repos'
import type { ScriptPreset } from '@/types/script'

const scriptsStore = useScriptsStore()
const reposStore = useReposStore()

const command = ref('')
const selectedRepoIds = ref<string[]>([])
const selectedPresetId = ref<string | null>(null)
const expandedRows = ref<Record<string, boolean>>({})
const showSaveModal = ref(false)
const newPresetName = ref('')
const newPresetDescription = ref('')

const selectedPreset = computed(() => {
  if (!selectedPresetId.value) return null
  return scriptsStore.presets.find(p => p.id === selectedPresetId.value) ?? null
})

const canRun = computed(() => {
  return command.value.trim().length > 0 && selectedRepoIds.value.length > 0 && !scriptsStore.isRunning
})

onMounted(() => {
  scriptsStore.loadPresets().catch(() => {})
  reposStore.loadRepos().catch(() => {})
})

function onPresetChange() {
  const preset = selectedPreset.value
  if (preset) {
    command.value = preset.command
  }
}

function toggleRepo(repoId: string) {
  const idx = selectedRepoIds.value.indexOf(repoId)
  if (idx !== -1) {
    selectedRepoIds.value.splice(idx, 1)
  } else {
    selectedRepoIds.value.push(repoId)
  }
}

function selectAllRepos() {
  selectedRepoIds.value = reposStore.repos.map(r => r.id)
}

function deselectAllRepos() {
  selectedRepoIds.value = []
}

async function handleRun() {
  if (!canRun.value) return
  await scriptsStore.runScriptAction(command.value, selectedRepoIds.value)
}

async function handleAbort() {
  await scriptsStore.abortScriptAction()
}

function toggleRow(repoId: string) {
  expandedRows.value[repoId] = !expandedRows.value[repoId]
}

function openSaveModal() {
  newPresetName.value = ''
  newPresetDescription.value = ''
  showSaveModal.value = true
}

async function handleSavePreset() {
  if (!newPresetName.value.trim()) return
  const preset: ScriptPreset = {
    id: crypto.randomUUID(),
    name: newPresetName.value.trim(),
    command: command.value,
    description: newPresetDescription.value.trim(),
  }
  await scriptsStore.savePresetAction(preset)
  showSaveModal.value = false
}

async function handleDeletePreset(id: string) {
  await scriptsStore.deletePresetAction(id)
  if (selectedPresetId.value === id) {
    selectedPresetId.value = null
  }
}

function exitCodeClass(code: number): string {
  return code === 0 ? 'text-success' : 'text-danger'
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-semibold">Script Runner</h1>

    <p v-if="scriptsStore.error" class="text-danger text-sm">{{ scriptsStore.error }}</p>

    <!-- Preset selector -->
    <div class="flex items-center gap-3">
      <select
        v-model="selectedPresetId"
        class="flex-1 rounded-md bg-surface-alt border border-border px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-primary"
        @change="onPresetChange"
      >
        <option :value="null">-- Select a preset --</option>
        <option v-for="preset in scriptsStore.presets" :key="preset.id" :value="preset.id">
          {{ preset.name }}
        </option>
      </select>
      <button
        class="px-3 py-2 rounded-md bg-surface-alt border border-border text-sm text-muted hover:text-white transition-colors"
        @click="openSaveModal"
      >
        Save as Preset
      </button>
      <button
        v-if="selectedPresetId"
        class="px-3 py-2 rounded-md bg-danger/20 text-danger text-sm hover:bg-danger/30 transition-colors"
        @click="handleDeletePreset(selectedPresetId)"
      >
        Delete
      </button>
    </div>

    <!-- Command input -->
    <div>
      <label class="block text-sm text-muted mb-1">Command</label>
      <textarea
        v-model="command"
        rows="4"
        class="w-full rounded-md bg-surface-alt border border-border px-3 py-2 font-mono text-sm text-white placeholder-muted focus:outline-none focus:ring-1 focus:ring-primary resize-y"
        placeholder="e.g. npm outdated --json"
      />
    </div>

    <!-- Repo multi-select -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <label class="text-sm text-muted">
          Target Repositories ({{ selectedRepoIds.length }} selected)
        </label>
        <div class="flex gap-2">
          <button class="text-xs text-primary hover:underline" @click="selectAllRepos">Select All</button>
          <button class="text-xs text-muted hover:underline" @click="deselectAllRepos">Deselect All</button>
        </div>
      </div>
      <div class="max-h-48 overflow-y-auto rounded-md bg-surface-alt border border-border p-2 space-y-1">
        <div v-if="reposStore.repos.length === 0" class="text-sm text-muted px-2 py-1">
          No repos loaded. Discover repos first.
        </div>
        <label
          v-for="repo in reposStore.repos"
          :key="repo.id"
          class="flex items-center gap-2 px-2 py-1 rounded hover:bg-surface cursor-pointer"
        >
          <input
            type="checkbox"
            :checked="selectedRepoIds.includes(repo.id)"
            class="rounded border-border"
            @change="toggleRepo(repo.id)"
          />
          <span class="text-sm font-mono">{{ repo.fullName }}</span>
        </label>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex gap-3">
      <button
        class="px-4 py-2 rounded-md text-sm font-medium transition-colors"
        :class="canRun
          ? 'bg-primary text-white hover:bg-primary/80'
          : 'bg-surface-alt text-muted border border-border cursor-not-allowed'"
        :disabled="!canRun"
        @click="handleRun"
      >
        {{ scriptsStore.isRunning ? 'Running...' : 'Run Script' }}
      </button>
      <button
        v-if="scriptsStore.isRunning"
        class="px-4 py-2 rounded-md bg-danger/20 text-danger text-sm font-medium hover:bg-danger/30 transition-colors"
        @click="handleAbort"
      >
        Abort
      </button>
    </div>

    <!-- Results -->
    <div v-if="scriptsStore.activeRun" class="space-y-3">
      <div class="flex items-center gap-3">
        <h2 class="text-lg font-medium">Results</h2>
        <span
          class="inline-block rounded px-2 py-0.5 text-xs font-mono"
          :class="{
            'bg-success/20 text-success': scriptsStore.activeRun.status === 'completed',
            'bg-warning/20 text-warning': scriptsStore.activeRun.status === 'running',
            'bg-danger/20 text-danger': scriptsStore.activeRun.status === 'aborted',
          }"
        >
          {{ scriptsStore.activeRun.status }}
        </span>
      </div>

      <div class="rounded-lg bg-surface-alt border border-border overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted">
              <th class="px-4 py-2 font-medium">Repository</th>
              <th class="px-4 py-2 font-medium">Exit Code</th>
              <th class="px-4 py-2 font-medium">Duration</th>
              <th class="px-4 py-2 font-medium text-right">Details</th>
            </tr>
          </thead>
          <tbody>
            <template v-for="result in scriptsStore.activeRun.results" :key="result.repoId">
              <tr
                class="border-b border-border last:border-b-0 hover:bg-surface/50 cursor-pointer"
                @click="toggleRow(result.repoId)"
              >
                <td class="px-4 py-2 font-mono text-xs">{{ result.repoId }}</td>
                <td class="px-4 py-2">
                  <span class="font-mono text-xs font-bold" :class="exitCodeClass(result.exitCode)">
                    {{ result.exitCode }}
                  </span>
                </td>
                <td class="px-4 py-2 text-xs text-muted">{{ result.durationMs }}ms</td>
                <td class="px-4 py-2 text-right text-xs text-muted">
                  {{ expandedRows[result.repoId] ? 'Hide' : 'Show' }}
                </td>
              </tr>
              <tr v-if="expandedRows[result.repoId]">
                <td colspan="4" class="px-4 py-3 bg-surface">
                  <div v-if="result.stdout" class="mb-2">
                    <span class="text-xs text-muted block mb-1">stdout</span>
                    <pre class="text-xs font-mono text-white whitespace-pre-wrap bg-surface-alt rounded p-2 max-h-48 overflow-y-auto">{{ result.stdout }}</pre>
                  </div>
                  <div v-if="result.stderr">
                    <span class="text-xs text-muted block mb-1">stderr</span>
                    <pre class="text-xs font-mono text-danger whitespace-pre-wrap bg-surface-alt rounded p-2 max-h-48 overflow-y-auto">{{ result.stderr }}</pre>
                  </div>
                  <div v-if="!result.stdout && !result.stderr" class="text-xs text-muted">
                    No output
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Save preset modal -->
    <Teleport to="body">
      <div
        v-if="showSaveModal"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
        @click.self="showSaveModal = false"
      >
        <div class="bg-surface-alt border border-border rounded-lg p-6 w-full max-w-md space-y-4">
          <h3 class="text-lg font-medium">Save Preset</h3>
          <div>
            <label class="block text-sm text-muted mb-1">Name</label>
            <input
              v-model="newPresetName"
              type="text"
              class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-primary"
              placeholder="My preset"
            />
          </div>
          <div>
            <label class="block text-sm text-muted mb-1">Description</label>
            <input
              v-model="newPresetDescription"
              type="text"
              class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-primary"
              placeholder="Optional description"
            />
          </div>
          <div class="flex justify-end gap-3">
            <button
              class="px-4 py-2 rounded-md text-sm text-muted hover:text-white transition-colors"
              @click="showSaveModal = false"
            >
              Cancel
            </button>
            <button
              class="px-4 py-2 rounded-md bg-primary text-white text-sm font-medium hover:bg-primary/80 transition-colors"
              :disabled="!newPresetName.trim()"
              @click="handleSavePreset"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
