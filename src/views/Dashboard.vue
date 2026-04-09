<script setup lang="ts">
import { onMounted } from 'vue'
import { useReposStore } from '@/stores/repos'
import { useScansStore } from '@/stores/scans'
import { useCveStore } from '@/stores/cve'
import { useOperationsStore } from '@/stores/operations'
import { useSettingsStore } from '@/stores/settings'

const reposStore = useReposStore()
const scansStore = useScansStore()
const cveStore = useCveStore()
const operationsStore = useOperationsStore()
const settingsStore = useSettingsStore()

onMounted(() => {
  Promise.all([
    reposStore.loadRepos(),
    scansStore.loadResults(),
    cveStore.loadAlerts(),
    operationsStore.loadOperations(),
    settingsStore.loadAuditLog(10),
    settingsStore.refreshRateLimit(),
  ]).catch(() => {
    // errors captured in store
  })
})

function formatTimestamp(ts: unknown): string {
  if (typeof ts !== 'string') return '-'
  const d = new Date(ts)
  if (isNaN(d.getTime())) return String(ts)
  return d.toLocaleString()
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-semibold">Dashboard</h1>

    <!-- Stat cards -->
    <div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
      <!-- Repos -->
      <div class="rounded-lg bg-surface-alt border border-border p-4">
        <p class="text-sm text-muted">Repositories</p>
        <p class="text-3xl font-bold mt-1">{{ reposStore.repos.length }}</p>
      </div>

      <!-- Health score -->
      <div class="rounded-lg bg-surface-alt border border-border p-4">
        <p class="text-sm text-muted">Avg Health Score</p>
        <p class="text-3xl font-bold mt-1" :class="{
          'text-success': scansStore.averageHealthScore >= 70,
          'text-warning': scansStore.averageHealthScore >= 40 && scansStore.averageHealthScore < 70,
          'text-danger': scansStore.averageHealthScore < 40 && scansStore.averageHealthScore > 0,
        }">
          {{ scansStore.averageHealthScore }}
        </p>
      </div>

      <!-- CVE alerts -->
      <div class="rounded-lg bg-surface-alt border border-border p-4">
        <p class="text-sm text-muted">CVE Alerts</p>
        <p class="text-3xl font-bold mt-1" :class="{ 'text-danger': cveStore.badgeCount > 0 }">
          {{ cveStore.alerts.length }}
        </p>
        <p v-if="cveStore.criticalCount > 0" class="text-xs text-danger mt-1">
          {{ cveStore.criticalCount }} critical
        </p>
      </div>

      <!-- Operations -->
      <div class="rounded-lg bg-surface-alt border border-border p-4">
        <p class="text-sm text-muted">Operations</p>
        <p class="text-3xl font-bold mt-1">{{ operationsStore.operations.length }}</p>
        <p v-if="operationsStore.runningOps.length > 0" class="text-xs text-primary mt-1">
          {{ operationsStore.runningOps.length }} running
        </p>
      </div>

      <!-- Rate limit -->
      <div class="rounded-lg bg-surface-alt border border-border p-4">
        <p class="text-sm text-muted">API Rate Limit</p>
        <template v-if="settingsStore.rateLimitGithub">
          <p class="text-3xl font-bold mt-1" :class="{
            'text-success': settingsStore.rateLimitGithub.remaining > 1000,
            'text-warning': settingsStore.rateLimitGithub.remaining <= 1000 && settingsStore.rateLimitGithub.remaining > 100,
            'text-danger': settingsStore.rateLimitGithub.remaining <= 100,
          }">
            {{ settingsStore.rateLimitGithub.remaining }}
          </p>
          <p class="text-xs text-muted mt-1">
            / {{ settingsStore.rateLimitGithub.limit }}
          </p>
        </template>
        <p v-else class="text-sm text-muted mt-2">No data</p>
      </div>
    </div>

    <!-- Recent activity -->
    <div>
      <h2 class="text-lg font-semibold mb-3">Recent Activity</h2>
      <div v-if="settingsStore.auditLog.length === 0"
           class="rounded-lg bg-surface-alt border border-border p-6 text-center text-muted">
        No recent activity
      </div>
      <div v-else class="rounded-lg bg-surface-alt border border-border overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted">
              <th class="px-4 py-2 font-medium">Timestamp</th>
              <th class="px-4 py-2 font-medium">Action</th>
              <th class="px-4 py-2 font-medium">Details</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(entry, idx) in settingsStore.auditLog"
                :key="idx"
                class="border-b border-border last:border-b-0 hover:bg-surface/50">
              <td class="px-4 py-2 font-mono text-xs whitespace-nowrap">
                {{ formatTimestamp(entry.timestamp) }}
              </td>
              <td class="px-4 py-2">
                <span class="inline-block rounded bg-surface px-2 py-0.5 text-xs font-mono border border-border">
                  {{ entry.action_type ?? entry.actionType ?? '-' }}
                </span>
              </td>
              <td class="px-4 py-2 text-muted truncate max-w-xs">
                {{ entry.summary ?? entry.details ?? '-' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
