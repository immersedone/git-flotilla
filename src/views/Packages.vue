<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { usePackagesStore } from '@/stores/packages'
import { useRepoListsStore } from '@/stores/repoLists'
import type { PackageRow } from '@/types/package'

const packagesStore = usePackagesStore()
const repoListsStore = useRepoListsStore()

const expandedPackage = ref<string | null>(null)
const sortBy = ref<'name' | 'repoCount'>('name')
const sortAsc = ref(true)
const changelogTarget = ref<string | null>(null)

const sortedPackages = computed(() => {
  const copy = [...packagesStore.filteredPackages]
  const dir = sortAsc.value ? 1 : -1
  return copy.sort((a, b) => {
    if (sortBy.value === 'repoCount') {
      return (a.repoCount - b.repoCount) * dir
    }
    return a.name.localeCompare(b.name) * dir
  })
})

function toggleSort(column: 'name' | 'repoCount') {
  if (sortBy.value === column) {
    sortAsc.value = !sortAsc.value
  } else {
    sortBy.value = column
    sortAsc.value = true
  }
}

function sortIndicator(column: 'name' | 'repoCount'): string {
  if (sortBy.value !== column) return ''
  return sortAsc.value ? ' \u2191' : ' \u2193'
}

function toggleExpand(pkg: PackageRow) {
  const key = `${pkg.ecosystem}:${pkg.name}`
  if (expandedPackage.value === key) {
    expandedPackage.value = null
    changelogTarget.value = null
  } else {
    expandedPackage.value = key
    changelogTarget.value = null
    packagesStore.changelog = []
  }
}

function isExpanded(pkg: PackageRow): boolean {
  return expandedPackage.value === `${pkg.ecosystem}:${pkg.name}`
}

function getVersionEntries(pkg: PackageRow): Array<{ repoId: string; version: string }> {
  return Object.entries(pkg.versionsByRepo).map(([repoId, version]) => ({ repoId, version }))
}

function versionColor(version: string, allVersions: string[]): string {
  const unique = [...new Set(allVersions)]
  if (unique.length <= 1) return 'text-green-500'
  // Most common version gets green (likely the "standard"), others get amber
  const counts: Record<string, number> = {}
  for (const v of allVersions) {
    counts[v] = (counts[v] ?? 0) + 1
  }
  const mostCommon = Object.entries(counts).sort((a, b) => b[1] - a[1])[0][0]
  return version === mostCommon ? 'text-green-500' : 'text-amber-500'
}

function compareSemver(a: string, b: string): number {
  const pa = a.replace(/^[^0-9]*/, '').split('.').map(Number)
  const pb = b.replace(/^[^0-9]*/, '').split('.').map(Number)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const diff = (pa[i] ?? 0) - (pb[i] ?? 0)
    if (diff !== 0) return diff
  }
  return 0
}

async function loadChangelog(pkg: PackageRow) {
  const versions = Object.values(pkg.versionsByRepo)
  const sorted = [...new Set(versions)].sort(compareSemver)
  const from = sorted[0]
  const to = pkg.latestVersion ?? sorted[sorted.length - 1]
  changelogTarget.value = `${pkg.ecosystem}:${pkg.name}`
  await packagesStore.fetchChangelog(pkg.name, pkg.ecosystem, from, to)
}

async function handleRepoListChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const value = target.value || null
  packagesStore.selectedRepoListId = value
  await packagesStore.loadMatrix(value ?? undefined)
}

async function handleExport() {
  const csv = await packagesStore.exportCsv(packagesStore.selectedRepoListId ?? undefined)
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'dependency-matrix.csv'
  a.click()
  URL.revokeObjectURL(url)
}

function selectEcosystem(eco: string | null) {
  packagesStore.selectedEcosystem = eco
}

function repoShortName(repoId: string): string {
  // "github:org/name" -> "org/name"
  const idx = repoId.indexOf(':')
  return idx !== -1 ? repoId.slice(idx + 1) : repoId
}

onMounted(() => {
  Promise.all([
    repoListsStore.loadLists(),
    packagesStore.loadMatrix(),
  ])
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold text-white">Packages</h1>
      <button
        :disabled="!packagesStore.matrix"
        class="px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        @click="handleExport"
      >
        Export CSV
      </button>
    </div>

    <!-- Filter bar -->
    <div class="bg-surface-alt border border-border rounded-lg p-4 space-y-3">
      <div class="flex flex-wrap items-center gap-4">
        <!-- Repo list dropdown -->
        <select
          class="bg-surface border border-border rounded px-3 py-2 text-sm text-white"
          :value="packagesStore.selectedRepoListId ?? ''"
          @change="handleRepoListChange"
        >
          <option value="">All repos</option>
          <option
            v-for="list in repoListsStore.lists"
            :key="list.id"
            :value="list.id"
          >
            {{ list.name }}
          </option>
        </select>

        <!-- Ecosystem pills -->
        <div class="flex gap-1">
          <button
            class="px-3 py-1 text-xs font-medium rounded-full border transition-colors"
            :class="packagesStore.selectedEcosystem === null
              ? 'bg-blue-500/20 border-blue-500/50 text-blue-400'
              : 'border-border text-gray-400 hover:text-gray-300'"
            @click="selectEcosystem(null)"
          >
            All
          </button>
          <button
            v-for="eco in packagesStore.ecosystems"
            :key="eco"
            class="px-3 py-1 text-xs font-medium rounded-full border transition-colors"
            :class="packagesStore.selectedEcosystem === eco
              ? 'bg-blue-500/20 border-blue-500/50 text-blue-400'
              : 'border-border text-gray-400 hover:text-gray-300'"
            @click="selectEcosystem(eco)"
          >
            {{ eco }}
          </button>
        </div>

        <!-- Search input -->
        <input
          v-model="packagesStore.searchQuery"
          type="text"
          placeholder="Search packages..."
          class="bg-surface border border-border rounded px-3 py-2 text-sm text-white placeholder-gray-500 flex-1 min-w-50"
        />
      </div>
    </div>

    <!-- Error display -->
    <div
      v-if="packagesStore.error"
      class="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 text-sm"
    >
      {{ packagesStore.error }}
    </div>

    <!-- Loading state -->
    <div
      v-if="packagesStore.isLoading"
      class="bg-surface-alt border border-border rounded-lg p-12 text-center"
    >
      <div class="text-gray-400 text-sm">Loading dependency matrix...</div>
    </div>

    <!-- Summary stats -->
    <div v-if="packagesStore.matrix && !packagesStore.isLoading" class="grid grid-cols-3 gap-4">
      <div class="bg-surface-alt border border-border rounded-lg p-4">
        <div class="text-sm text-gray-500">Total Packages</div>
        <div class="text-2xl font-semibold text-white mt-1">
          {{ packagesStore.matrix.packages.length }}
        </div>
      </div>
      <div class="bg-surface-alt border border-border rounded-lg p-4">
        <div class="text-sm text-gray-500">Packages with Drift</div>
        <div class="text-2xl font-semibold text-amber-500 mt-1">
          {{ packagesStore.driftCount }}
        </div>
      </div>
      <div class="bg-surface-alt border border-border rounded-lg p-4">
        <div class="text-sm text-gray-500">Ecosystems</div>
        <div class="text-2xl font-semibold text-white mt-1">
          {{ packagesStore.ecosystems.length }}
        </div>
      </div>
    </div>

    <!-- Matrix table -->
    <div
      v-if="packagesStore.matrix && sortedPackages.length > 0 && !packagesStore.isLoading"
      class="bg-surface-alt border border-border rounded-lg overflow-hidden"
    >
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border text-left text-gray-500">
            <th
              class="px-4 py-3 cursor-pointer hover:text-gray-300"
              @click="toggleSort('name')"
            >
              Package{{ sortIndicator('name') }}
            </th>
            <th class="px-4 py-3">Ecosystem</th>
            <th
              class="px-4 py-3 cursor-pointer hover:text-gray-300"
              @click="toggleSort('repoCount')"
            >
              Repos{{ sortIndicator('repoCount') }}
            </th>
            <th class="px-4 py-3">Drift</th>
            <th class="px-4 py-3">Dev Only</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="pkg in sortedPackages" :key="`${pkg.ecosystem}:${pkg.name}`">
            <tr
              class="border-b border-border hover:bg-surface cursor-pointer transition-colors"
              :class="{ 'border-l-2 border-l-amber-500': pkg.hasDrift }"
              @click="toggleExpand(pkg)"
            >
              <td class="px-4 py-3 font-mono text-white">{{ pkg.name }}</td>
              <td class="px-4 py-3 text-gray-400">{{ pkg.ecosystem }}</td>
              <td class="px-4 py-3 text-gray-300">{{ pkg.repoCount }}</td>
              <td class="px-4 py-3">
                <span
                  v-if="pkg.hasDrift"
                  class="inline-block px-2 py-0.5 rounded text-xs font-medium bg-amber-500/10 text-amber-500 border border-amber-500/30"
                >
                  Yes
                </span>
                <span
                  v-else
                  class="inline-block px-2 py-0.5 rounded text-xs font-medium bg-green-500/10 text-green-500 border border-green-500/30"
                >
                  No
                </span>
              </td>
              <td class="px-4 py-3">
                <span
                  v-if="pkg.isDevOnly"
                  class="inline-block px-2 py-0.5 rounded text-xs font-medium bg-gray-500/10 text-gray-400 border border-gray-500/30"
                >
                  Dev
                </span>
              </td>
            </tr>

            <!-- Expanded row -->
            <tr v-if="isExpanded(pkg)">
              <td colspan="5" class="bg-surface px-6 py-4 border-b border-border">
                <div class="space-y-4">
                  <!-- Per-repo version breakdown -->
                  <div>
                    <h4 class="text-gray-400 font-medium text-xs uppercase tracking-wide mb-2">
                      Version by Repository
                    </h4>
                    <div class="space-y-1">
                      <div
                        v-for="entry in getVersionEntries(pkg)"
                        :key="entry.repoId"
                        class="flex items-center gap-3 text-sm"
                      >
                        <span class="font-mono text-gray-400 min-w-50">
                          {{ repoShortName(entry.repoId) }}
                        </span>
                        <span
                          class="font-mono"
                          :class="pkg.hasDrift
                            ? versionColor(entry.version, Object.values(pkg.versionsByRepo))
                            : 'text-green-500'"
                        >
                          {{ entry.version }}
                        </span>
                      </div>
                    </div>
                    <div v-if="pkg.latestVersion" class="mt-2 text-xs text-gray-500">
                      Latest available: <span class="font-mono text-gray-300">{{ pkg.latestVersion }}</span>
                    </div>
                  </div>

                  <!-- Changelog section -->
                  <div v-if="pkg.ecosystem === 'npm'">
                    <button
                      v-if="changelogTarget !== `${pkg.ecosystem}:${pkg.name}`"
                      class="px-3 py-1.5 text-xs font-medium text-blue-400 border border-blue-400/30 rounded hover:bg-blue-400/10 transition-colors"
                      @click.stop="loadChangelog(pkg)"
                    >
                      View Changelog
                    </button>

                    <div v-if="changelogTarget === `${pkg.ecosystem}:${pkg.name}`">
                      <div v-if="packagesStore.changelogLoading" class="text-xs text-gray-500 py-2">
                        Loading changelog...
                      </div>
                      <div v-else-if="packagesStore.changelog.length > 0" class="space-y-3 mt-2">
                        <div
                          v-for="entry in packagesStore.changelog"
                          :key="entry.version"
                          class="border border-border rounded p-3"
                        >
                          <div class="flex items-center gap-2 mb-1">
                            <span class="font-mono text-white text-sm font-medium">
                              {{ entry.version }}
                            </span>
                            <span
                              v-if="entry.isBreaking"
                              class="inline-block px-2 py-0.5 rounded text-xs font-medium bg-red-500/10 text-red-500 border border-red-500/30"
                            >
                              Breaking
                            </span>
                            <span class="text-xs text-gray-500">
                              {{ entry.publishedAt }}
                            </span>
                          </div>
                          <pre class="text-xs text-gray-400 whitespace-pre-wrap font-mono">{{ entry.body }}</pre>
                        </div>
                      </div>
                      <div v-else class="text-xs text-gray-500 py-2">
                        No changelog entries found.
                      </div>
                    </div>
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
      v-if="!packagesStore.isLoading && (!packagesStore.matrix || packagesStore.matrix.packages.length === 0)"
      class="bg-surface-alt border border-border rounded-lg p-12 text-center"
    >
      <div class="text-gray-500 text-lg mb-2">No packages found</div>
      <p class="text-gray-600 text-sm">Run a scan first to build the dependency matrix.</p>
    </div>

    <!-- Filtered empty state -->
    <div
      v-if="!packagesStore.isLoading && packagesStore.matrix && packagesStore.matrix.packages.length > 0 && sortedPackages.length === 0"
      class="bg-surface-alt border border-border rounded-lg p-8 text-center"
    >
      <div class="text-gray-500 text-sm">No packages match your filters.</div>
    </div>
  </div>
</template>
