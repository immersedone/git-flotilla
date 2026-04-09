<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useScansStore } from '@/stores/scans'
import { useReposStore } from '@/stores/repos'
import { useRepoListsStore } from '@/stores/repoLists'
import type { ScanResult } from '@/types/scan'

const scansStore = useScansStore()
const reposStore = useReposStore()
const repoListsStore = useRepoListsStore()

const selectedRepoId = ref<string | null>(null)
const selectedListId = ref<string | null>(null)
const expandedRepoId = ref<string | null>(null)
const sortColumn = ref<keyof ScanResult | 'issues'>('repoId')
const sortDirection = ref<'asc' | 'desc'>('asc')

const reposWithIssues = computed(() =>
  scansStore.results.filter(r => r.flags.length > 0).length,
)

const progressPercent = computed(() => {
  if (scansStore.scanProgress.total === 0) return 0
  return Math.round((scansStore.scanProgress.current / scansStore.scanProgress.total) * 100)
})

const sortedResults = computed(() => {
  const copy = [...scansStore.results]
  const dir = sortDirection.value === 'asc' ? 1 : -1
  const col = sortColumn.value

  return copy.sort((a, b) => {
    if (col === 'issues') {
      return (a.flags.length - b.flags.length) * dir
    }
    if (col === 'healthScore') {
      return (a.healthScore - b.healthScore) * dir
    }
    const aVal = String(a[col] ?? '')
    const bVal = String(b[col] ?? '')
    return aVal.localeCompare(bVal) * dir
  })
})

function toggleSort(column: keyof ScanResult | 'issues') {
  if (sortColumn.value === column) {
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = column
    sortDirection.value = 'asc'
  }
}

function sortIndicator(column: keyof ScanResult | 'issues'): string {
  if (sortColumn.value !== column) return ''
  return sortDirection.value === 'asc' ? ' \u2191' : ' \u2193'
}

function healthColor(score: number): string {
  if (score >= 80) return 'text-green-500'
  if (score >= 50) return 'text-amber-500'
  return 'text-red-500'
}

function healthBgColor(score: number): string {
  if (score >= 80) return 'bg-green-500/10 text-green-500 border-green-500/30'
  if (score >= 50) return 'bg-amber-500/10 text-amber-500 border-amber-500/30'
  return 'bg-red-500/10 text-red-500 border-red-500/30'
}

function severityDot(severity: string): string {
  switch (severity) {
    case 'critical': return 'bg-red-500'
    case 'high': return 'bg-orange-500'
    case 'medium': return 'bg-amber-500'
    case 'low': return 'bg-blue-500'
    default: return 'bg-gray-500'
  }
}

function repoName(repoId: string): string {
  const repo = reposStore.repos.find(r => r.id === repoId)
  return repo?.fullName ?? repoId
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-AU', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function toggleExpand(repoId: string) {
  expandedRepoId.value = expandedRepoId.value === repoId ? null : repoId
}

async function scanSingle() {
  if (!selectedRepoId.value) return
  await scansStore.scanSingleRepo(selectedRepoId.value)
}

async function scanAll() {
  if (!selectedListId.value) return
  await scansStore.scanList(selectedListId.value)
}

onMounted(() => {
  Promise.all([
    reposStore.loadRepos(),
    repoListsStore.loadLists(),
    scansStore.loadResults(),
  ])
})
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-semibold text-white">Scanner</h1>

    <!-- Scan Controls -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Single repo scan -->
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <h3 class="text-sm font-medium text-gray-400 mb-3">Scan Single Repo</h3>
        <div class="flex gap-2">
          <select
            v-model="selectedRepoId"
            :disabled="scansStore.isScanning"
            class="flex-1 bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2 text-sm text-white font-mono disabled:opacity-50"
          >
            <option :value="null">Select a repository...</option>
            <option v-for="repo in reposStore.repos" :key="repo.id" :value="repo.id">
              {{ repo.fullName }}
            </option>
          </select>
          <button
            :disabled="!selectedRepoId || scansStore.isScanning"
            class="px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
            @click="scanSingle"
          >
            Scan
          </button>
        </div>
      </div>

      <!-- Repo list scan -->
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <h3 class="text-sm font-medium text-gray-400 mb-3">Scan Repo List</h3>
        <div class="flex gap-2">
          <select
            v-model="selectedListId"
            :disabled="scansStore.isScanning"
            class="flex-1 bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2 text-sm text-white disabled:opacity-50"
          >
            <option :value="null">Select a repo list...</option>
            <option v-for="list in repoListsStore.lists" :key="list.id" :value="list.id">
              {{ list.name }} ({{ list.repoIds.length }} repos)
            </option>
          </select>
          <button
            :disabled="!selectedListId || scansStore.isScanning"
            class="px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
            @click="scanAll"
          >
            Scan All
          </button>
        </div>
      </div>
    </div>

    <!-- Progress bar -->
    <div
      v-if="scansStore.isScanning"
      class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4 space-y-3"
    >
      <div class="flex items-center justify-between">
        <span class="text-sm text-white">
          Scanning {{ scansStore.scanProgress.current }} / {{ scansStore.scanProgress.total }} repos
        </span>
        <button
          class="px-3 py-1 text-sm text-red-400 border border-red-400/30 rounded hover:bg-red-400/10"
          @click="scansStore.abortCurrentScan()"
        >
          Abort
        </button>
      </div>
      <div class="w-full bg-[#0F1117] rounded-full h-2 overflow-hidden">
        <div
          class="bg-blue-500 h-full rounded-full transition-all duration-300"
          :style="{ width: progressPercent + '%' }"
        />
      </div>
      <div class="flex gap-4 text-xs text-gray-500">
        <span>{{ scansStore.scanSummary.succeeded }} done</span>
        <span>{{ scansStore.scanSummary.failed }} failed</span>
        <span>{{ scansStore.scanSummary.inProgress }} scanning</span>
      </div>
    </div>

    <!-- Error display -->
    <div
      v-if="scansStore.error"
      class="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 text-sm"
    >
      {{ scansStore.error }}
    </div>

    <!-- Summary stats -->
    <div v-if="scansStore.results.length > 0" class="grid grid-cols-3 gap-4">
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">Repos Scanned</div>
        <div class="text-2xl font-semibold text-white mt-1">{{ scansStore.results.length }}</div>
      </div>
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">Average Health Score</div>
        <div class="text-2xl font-semibold mt-1" :class="healthColor(scansStore.averageHealthScore)">
          {{ scansStore.averageHealthScore }}
        </div>
      </div>
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">Repos With Issues</div>
        <div class="text-2xl font-semibold text-white mt-1">{{ reposWithIssues }}</div>
      </div>
    </div>

    <!-- Results table -->
    <div v-if="scansStore.results.length > 0" class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-[#2A2D3A] text-left text-gray-500">
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('repoId')">
              Repository{{ sortIndicator('repoId') }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('healthScore')">
              Health{{ sortIndicator('healthScore') }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('packageManager')">
              Package Manager{{ sortIndicator('packageManager') }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('nodeVersion')">
              Node{{ sortIndicator('nodeVersion') }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('manifestPaths')">
              Manifests{{ sortIndicator('manifestPaths') }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('issues')">
              Issues{{ sortIndicator('issues') }}
            </th>
            <th class="px-4 py-3 cursor-pointer hover:text-gray-300" @click="toggleSort('scannedAt')">
              Scanned{{ sortIndicator('scannedAt') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <template v-for="result in sortedResults" :key="result.repoId">
            <tr
              class="border-b border-[#2A2D3A] hover:bg-[#0F1117] cursor-pointer transition-colors"
              @click="toggleExpand(result.repoId)"
            >
              <td class="px-4 py-3 font-mono text-white">{{ repoName(result.repoId) }}</td>
              <td class="px-4 py-3">
                <span
                  class="inline-block px-2 py-0.5 rounded border text-xs font-medium"
                  :class="healthBgColor(result.healthScore)"
                >
                  {{ result.healthScore }}
                </span>
              </td>
              <td class="px-4 py-3 font-mono text-gray-300">
                <template v-if="result.packageManager">
                  {{ result.packageManager }}
                  <span v-if="result.packageManagerVersion" class="text-gray-500">
                    {{ result.packageManagerVersion }}
                  </span>
                </template>
                <span v-else class="text-gray-600">-</span>
              </td>
              <td class="px-4 py-3 font-mono text-gray-300">
                <template v-if="result.nodeVersion">
                  {{ result.nodeVersion }}
                  <span v-if="result.nodeVersionSource" class="text-gray-500 text-xs">
                    ({{ result.nodeVersionSource }})
                  </span>
                </template>
                <span v-else class="text-gray-600">-</span>
              </td>
              <td class="px-4 py-3 text-gray-300">{{ result.manifestPaths.length }}</td>
              <td class="px-4 py-3">
                <span v-if="result.flags.length > 0" class="text-amber-500">
                  {{ result.flags.length }}
                </span>
                <span v-else class="text-gray-600">0</span>
              </td>
              <td class="px-4 py-3 text-gray-500 text-xs">{{ formatDate(result.scannedAt) }}</td>
            </tr>

            <!-- Expanded detail row -->
            <tr v-if="expandedRepoId === result.repoId">
              <td colspan="7" class="bg-[#0F1117] px-6 py-4 border-b border-[#2A2D3A]">
                <div class="grid grid-cols-2 gap-6 text-sm">
                  <!-- Manifest paths -->
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Manifest Paths</h4>
                    <ul class="space-y-1">
                      <li
                        v-for="path in result.manifestPaths"
                        :key="path"
                        class="font-mono text-gray-300 text-xs"
                      >
                        {{ path }}
                      </li>
                      <li v-if="result.manifestPaths.length === 0" class="text-gray-600 text-xs">
                        None found
                      </li>
                    </ul>
                  </div>

                  <!-- Workflow files -->
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Workflow Files</h4>
                    <ul class="space-y-1">
                      <li
                        v-for="wf in result.workflowFiles"
                        :key="wf"
                        class="font-mono text-gray-300 text-xs"
                      >
                        {{ wf }}
                      </li>
                      <li v-if="result.workflowFiles.length === 0" class="text-gray-600 text-xs">
                        None found
                      </li>
                    </ul>
                  </div>

                  <!-- Issues / Flags -->
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Issues</h4>
                    <ul class="space-y-1">
                      <li
                        v-for="(flag, idx) in result.flags"
                        :key="idx"
                        class="flex items-center gap-2 text-xs text-gray-300"
                      >
                        <span class="w-2 h-2 rounded-full shrink-0" :class="severityDot(flag.severity)" />
                        {{ flag.message }}
                      </li>
                      <li v-if="result.flags.length === 0" class="text-gray-600 text-xs">
                        No issues detected
                      </li>
                    </ul>
                  </div>

                  <!-- Details -->
                  <div>
                    <h4 class="text-gray-400 font-medium mb-2">Details</h4>
                    <dl class="space-y-1 text-xs">
                      <div class="flex gap-2">
                        <dt class="text-gray-500">PHP Version:</dt>
                        <dd class="font-mono text-gray-300">{{ result.phpVersion ?? '-' }}</dd>
                      </div>
                      <div class="flex gap-2">
                        <dt class="text-gray-500">Develop Branch:</dt>
                        <dd class="text-gray-300">{{ result.hasDevelop ? 'Yes' : 'No' }}</dd>
                      </div>
                      <div class="flex gap-2">
                        <dt class="text-gray-500">.env.example:</dt>
                        <dd class="text-gray-300">{{ result.hasDotEnvExample ? 'Present' : 'Missing' }}</dd>
                      </div>
                      <div v-if="result.excluded" class="flex gap-2">
                        <dt class="text-gray-500">Excluded:</dt>
                        <dd class="text-amber-500">{{ result.excludeReason ?? 'Yes' }}</dd>
                      </div>
                      <div v-if="result.lastPushed" class="flex gap-2">
                        <dt class="text-gray-500">Last Pushed:</dt>
                        <dd class="text-gray-300">{{ formatDate(result.lastPushed) }}</dd>
                      </div>
                    </dl>
                  </div>
                </div>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div
      v-if="scansStore.results.length === 0 && !scansStore.isScanning"
      class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-12 text-center"
    >
      <div class="text-gray-500 text-lg mb-2">No scan results yet</div>
      <p class="text-gray-600 text-sm">
        Select a repository or repo list above to start scanning.
      </p>
    </div>
  </div>
</template>
