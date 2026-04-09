<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useCveStore } from '@/stores/cve'
import type { CveSeverity, CveStatus } from '@/types/cve'

const cveStore = useCveStore()

const searchQuery = ref('')
const expandedCveId = ref<string | null>(null)
const watchlistPackage = ref('')
const watchlistEcosystem = ref('npm')

const severityOptions: Array<{ label: string; value: CveSeverity | null }> = [
  { label: 'All', value: null },
  { label: 'Critical', value: 'critical' },
  { label: 'High', value: 'high' },
  { label: 'Medium', value: 'medium' },
  { label: 'Low', value: 'low' },
]

const statusOptions: Array<{ label: string; value: CveStatus | null }> = [
  { label: 'All', value: null },
  { label: 'New', value: 'new' },
  { label: 'Acknowledged', value: 'acknowledged' },
  { label: 'Dismissed', value: 'dismissed' },
]

const ecosystemOptions = ['npm', 'composer', 'pip', 'cargo', 'go']

const displayedAlerts = computed(() => {
  const q = searchQuery.value.toLowerCase()
  if (!q) return cveStore.filteredAlerts
  return cveStore.filteredAlerts.filter(
    (a) => a.id.toLowerCase().includes(q) || a.packageName.toLowerCase().includes(q),
  )
})

const totalCount = computed(() => cveStore.alerts.length)
const criticalCount = computed(() => cveStore.alerts.filter((a) => a.severity === 'critical').length)
const highCount = computed(() => cveStore.alerts.filter((a) => a.severity === 'high').length)
const acknowledgedCount = computed(() => cveStore.alerts.filter((a) => a.status === 'acknowledged').length)

function severityBadgeClass(severity: CveSeverity): string {
  switch (severity) {
    case 'critical':
      return 'bg-red-500/10 text-red-500 border-red-500/30'
    case 'high':
      return 'bg-orange-500/10 text-orange-500 border-orange-500/30'
    case 'medium':
      return 'bg-amber-500/10 text-amber-500 border-amber-500/30'
    case 'low':
      return 'bg-blue-500/10 text-blue-500 border-blue-500/30'
  }
}

function severityPillClass(value: CveSeverity | null): string {
  const active = cveStore.selectedSeverity === value
  if (active) return 'bg-blue-500 text-white'
  return 'bg-[#0F1117] text-gray-400 hover:text-gray-200'
}

function statusPillClass(value: CveStatus | null): string {
  const active = cveStore.selectedStatus === value
  if (active) return 'bg-blue-500 text-white'
  return 'bg-[#0F1117] text-gray-400 hover:text-gray-200'
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-AU', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function toggleExpand(cveId: string) {
  expandedCveId.value = expandedCveId.value === cveId ? null : cveId
}

async function handleRunCheck() {
  try {
    await cveStore.runCheck()
  } catch {
    // error is captured in store
  }
}

async function handleAcknowledge(cveId: string) {
  try {
    await cveStore.acknowledgeAlert(cveId)
    expandedCveId.value = null
  } catch {
    // error is captured in store
  }
}

async function handleDismiss(cveId: string) {
  try {
    await cveStore.dismissAlert(cveId)
    expandedCveId.value = null
  } catch {
    // error is captured in store
  }
}

async function handleSnooze(cveId: string, repoId: string) {
  try {
    await cveStore.snoozeAlert(cveId, repoId, 7)
  } catch {
    // error is captured in store
  }
}

async function handleAddToWatchlist() {
  const pkg = watchlistPackage.value.trim()
  if (!pkg) return
  try {
    await cveStore.addToWatchlistAction(pkg, watchlistEcosystem.value)
    watchlistPackage.value = ''
  } catch {
    // error is captured in store
  }
}

async function handleRemoveFromWatchlist(packageName: string, ecosystem: string) {
  try {
    await cveStore.removeFromWatchlistAction(packageName, ecosystem)
  } catch {
    // error is captured in store
  }
}

onMounted(() => {
  Promise.all([cveStore.loadAlerts(), cveStore.loadWatchlist()]).catch(() => {
    // errors captured in store
  })
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold text-white">CVE Alerts</h1>
      <button
        :disabled="cveStore.isLoading"
        class="px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        @click="handleRunCheck"
      >
        <span v-if="cveStore.isLoading">Checking...</span>
        <span v-else>Check Now</span>
      </button>
    </div>

    <!-- Error display -->
    <div
      v-if="cveStore.error"
      class="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 text-sm"
    >
      {{ cveStore.error }}
    </div>

    <!-- Filter bar -->
    <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4 space-y-3">
      <div class="flex flex-wrap items-center gap-4">
        <!-- Severity pills -->
        <div class="flex items-center gap-1">
          <span class="text-xs text-gray-500 mr-1">Severity:</span>
          <button
            v-for="opt in severityOptions"
            :key="String(opt.value)"
            class="px-3 py-1 text-xs font-medium rounded border border-[#2A2D3A] transition-colors"
            :class="severityPillClass(opt.value)"
            @click="cveStore.selectedSeverity = opt.value"
          >
            {{ opt.label }}
          </button>
        </div>

        <!-- Status pills -->
        <div class="flex items-center gap-1">
          <span class="text-xs text-gray-500 mr-1">Status:</span>
          <button
            v-for="opt in statusOptions"
            :key="String(opt.value)"
            class="px-3 py-1 text-xs font-medium rounded border border-[#2A2D3A] transition-colors"
            :class="statusPillClass(opt.value)"
            @click="cveStore.selectedStatus = opt.value"
          >
            {{ opt.label }}
          </button>
        </div>

        <!-- Search -->
        <div class="flex-1 min-w-[200px]">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search CVE ID or package name..."
            class="w-full bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-1.5 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-blue-500"
          />
        </div>
      </div>
    </div>

    <!-- Summary stats -->
    <div class="grid grid-cols-4 gap-4">
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">Total Alerts</div>
        <div class="text-2xl font-semibold text-white mt-1">{{ totalCount }}</div>
      </div>
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">Critical</div>
        <div class="text-2xl font-semibold text-red-500 mt-1">{{ criticalCount }}</div>
      </div>
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">High</div>
        <div class="text-2xl font-semibold text-orange-500 mt-1">{{ highCount }}</div>
      </div>
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
        <div class="text-sm text-gray-500">Acknowledged</div>
        <div class="text-2xl font-semibold text-white mt-1">{{ acknowledgedCount }}</div>
      </div>
    </div>

    <!-- Alerts table -->
    <div
      v-if="displayedAlerts.length > 0"
      class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg overflow-hidden"
    >
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-[#2A2D3A] text-left text-gray-500">
            <th class="px-4 py-3">CVE ID</th>
            <th class="px-4 py-3">Package</th>
            <th class="px-4 py-3">Ecosystem</th>
            <th class="px-4 py-3">Severity</th>
            <th class="px-4 py-3">Affected Repos</th>
            <th class="px-4 py-3">Fixed Version</th>
            <th class="px-4 py-3">Status</th>
            <th class="px-4 py-3">Published</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="alert in displayedAlerts" :key="alert.id">
            <tr
              class="border-b border-[#2A2D3A] hover:bg-[#0F1117] cursor-pointer transition-colors"
              @click="toggleExpand(alert.id)"
            >
              <td class="px-4 py-3 font-mono text-white">{{ alert.id }}</td>
              <td class="px-4 py-3 font-mono text-gray-300">{{ alert.packageName }}</td>
              <td class="px-4 py-3 text-gray-300">{{ alert.ecosystem }}</td>
              <td class="px-4 py-3">
                <span
                  class="inline-block px-2 py-0.5 rounded border text-xs font-medium"
                  :class="severityBadgeClass(alert.severity)"
                >
                  {{ alert.severity }}
                </span>
              </td>
              <td class="px-4 py-3 text-gray-300">{{ alert.affectedRepos.length }}</td>
              <td class="px-4 py-3 font-mono text-gray-300">
                {{ alert.fixedVersion ?? '-' }}
              </td>
              <td class="px-4 py-3 text-gray-400">{{ alert.status }}</td>
              <td class="px-4 py-3 text-gray-500 text-xs">{{ formatDate(alert.publishedAt) }}</td>
            </tr>

            <!-- Expanded row -->
            <tr v-if="expandedCveId === alert.id">
              <td colspan="8" class="bg-[#0F1117] px-6 py-4 border-b border-[#2A2D3A]">
                <div class="space-y-4">
                  <!-- Summary -->
                  <div>
                    <h4 class="text-gray-400 text-xs font-medium mb-1">Summary</h4>
                    <p class="text-sm text-gray-300">{{ alert.summary }}</p>
                  </div>

                  <!-- Affected version range -->
                  <div>
                    <h4 class="text-gray-400 text-xs font-medium mb-1">Affected Versions</h4>
                    <span class="font-mono text-sm text-gray-300">{{ alert.affectedVersionRange }}</span>
                  </div>

                  <!-- Affected repos -->
                  <div>
                    <h4 class="text-gray-400 text-xs font-medium mb-1">Affected Repositories</h4>
                    <div class="flex flex-wrap gap-2">
                      <span
                        v-for="repoId in alert.affectedRepos"
                        :key="repoId"
                        class="inline-block px-2 py-0.5 bg-[#1A1D27] border border-[#2A2D3A] rounded text-xs font-mono text-gray-300"
                      >
                        {{ repoId }}
                      </span>
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="flex items-center gap-2 pt-2">
                    <button
                      v-if="alert.status === 'new'"
                      class="px-3 py-1.5 text-xs font-medium rounded bg-blue-500/10 text-blue-400 border border-blue-500/30 hover:bg-blue-500/20"
                      @click.stop="handleAcknowledge(alert.id)"
                    >
                      Acknowledge
                    </button>
                    <button
                      v-if="alert.status !== 'dismissed'"
                      class="px-3 py-1.5 text-xs font-medium rounded bg-gray-500/10 text-gray-400 border border-gray-500/30 hover:bg-gray-500/20"
                      @click.stop="handleDismiss(alert.id)"
                    >
                      Dismiss
                    </button>
                    <button
                      v-if="alert.affectedRepos.length > 0"
                      class="px-3 py-1.5 text-xs font-medium rounded bg-amber-500/10 text-amber-400 border border-amber-500/30 hover:bg-amber-500/20"
                      @click.stop="handleSnooze(alert.id, alert.affectedRepos[0])"
                    >
                      Snooze 7 days
                    </button>
                    <router-link
                      :to="'/cve/' + alert.id"
                      class="px-3 py-1.5 text-xs font-medium rounded bg-[#1A1D27] text-gray-300 border border-[#2A2D3A] hover:bg-[#2A2D3A]"
                      @click.stop
                    >
                      View Incident Timeline
                    </router-link>
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
      v-if="displayedAlerts.length === 0 && !cveStore.isLoading"
      class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-12 text-center"
    >
      <div class="text-gray-500 text-lg mb-2">No CVE alerts</div>
      <p class="text-gray-600 text-sm">
        Run a check to scan for vulnerabilities.
      </p>
    </div>

    <!-- Watchlist section -->
    <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4 space-y-4">
      <h2 class="text-lg font-semibold text-white">Watchlist</h2>

      <!-- Add form -->
      <div class="flex items-center gap-2">
        <input
          v-model="watchlistPackage"
          type="text"
          placeholder="Package name..."
          class="flex-1 bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2 text-sm text-white placeholder-gray-600 font-mono focus:outline-none focus:border-blue-500"
          @keyup.enter="handleAddToWatchlist"
        />
        <select
          v-model="watchlistEcosystem"
          class="bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2 text-sm text-white"
        >
          <option v-for="eco in ecosystemOptions" :key="eco" :value="eco">
            {{ eco }}
          </option>
        </select>
        <button
          :disabled="!watchlistPackage.trim()"
          class="px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
          @click="handleAddToWatchlist"
        >
          Add
        </button>
      </div>

      <!-- Watchlist items -->
      <div v-if="cveStore.watchlist.length > 0" class="space-y-1">
        <div
          v-for="item in cveStore.watchlist"
          :key="item.packageName + ':' + item.ecosystem"
          class="flex items-center justify-between bg-[#0F1117] border border-[#2A2D3A] rounded px-3 py-2"
        >
          <div class="flex items-center gap-3">
            <span class="font-mono text-sm text-white">{{ item.packageName }}</span>
            <span class="text-xs text-gray-500 bg-[#1A1D27] px-2 py-0.5 rounded border border-[#2A2D3A]">
              {{ item.ecosystem }}
            </span>
            <span class="text-xs text-gray-600">added {{ formatDate(item.addedAt) }}</span>
          </div>
          <button
            class="text-xs text-red-400 hover:text-red-300"
            @click="handleRemoveFromWatchlist(item.packageName, item.ecosystem)"
          >
            Remove
          </button>
        </div>
      </div>
      <div v-else class="text-sm text-gray-600">
        No packages in watchlist. Add packages to monitor for CVEs regardless of scan results.
      </div>
    </div>
  </div>
</template>
