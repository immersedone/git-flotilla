<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useScansStore } from '@/stores/scans'

const scansStore = useScansStore()

interface DriftRow {
  configKey: string
  mostCommonValue: string
  count: number
  total: number
  divergentRepos: { repoId: string; value: string }[]
}

function computeDrift(
  key: string,
  extract: (r: { repoId: string; nodeVersion: string | null; packageManager: string | null; packageManagerVersion: string | null }) => string | null,
): DriftRow | null {
  const entries: { repoId: string; value: string }[] = []

  for (const result of scansStore.results) {
    const val = extract(result)
    if (val !== null && val !== '') {
      entries.push({ repoId: result.repoId, value: val })
    }
  }

  if (entries.length === 0) return null

  const counts = new Map<string, number>()
  for (const entry of entries) {
    counts.set(entry.value, (counts.get(entry.value) ?? 0) + 1)
  }

  let mostCommon = ''
  let maxCount = 0
  for (const [value, count] of counts) {
    if (count > maxCount) {
      mostCommon = value
      maxCount = count
    }
  }

  const divergent = entries.filter(e => e.value !== mostCommon)

  return {
    configKey: key,
    mostCommonValue: mostCommon,
    count: maxCount,
    total: entries.length,
    divergentRepos: divergent,
  }
}

const driftRows = computed<DriftRow[]>(() => {
  const rows: DriftRow[] = []

  const nodeRow = computeDrift('Node Version', r => r.nodeVersion)
  if (nodeRow) rows.push(nodeRow)

  const pmRow = computeDrift('Package Manager', r => r.packageManager)
  if (pmRow) rows.push(pmRow)

  const pmVersionRow = computeDrift('Package Manager Version', r =>
    r.packageManager && r.packageManagerVersion
      ? `${r.packageManager}@${r.packageManagerVersion}`
      : null,
  )
  if (pmVersionRow) rows.push(pmVersionRow)

  return rows
})

const hasDrift = computed(() => driftRows.value.some(r => r.divergentRepos.length > 0))

onMounted(() => {
  if (scansStore.results.length === 0) {
    void scansStore.loadResults()
  }
})
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-semibold">Drift Dashboard</h1>

    <p v-if="scansStore.error" class="text-danger text-sm">{{ scansStore.error }}</p>

    <div v-if="scansStore.results.length === 0 && !scansStore.isScanning" class="rounded-lg bg-surface-alt border border-border p-8 text-center text-muted">
      <p class="text-lg font-medium mb-1">No scan results</p>
      <p class="text-sm">Run a scan first to detect configuration drift across repos.</p>
    </div>

    <div v-else-if="driftRows.length === 0 && !scansStore.isScanning" class="rounded-lg bg-surface-alt border border-border p-8 text-center text-muted">
      <p class="text-lg font-medium mb-1">No drift data available</p>
      <p class="text-sm">Scanned repos do not have comparable configuration values.</p>
    </div>

    <template v-else>
      <div v-if="!hasDrift" class="rounded-lg bg-success/10 border border-success/30 p-4">
        <p class="text-success text-sm font-medium">All repos are aligned. No configuration drift detected.</p>
      </div>

      <div class="rounded-lg bg-surface-alt border border-border overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted">
              <th class="px-4 py-2 font-medium">Configuration</th>
              <th class="px-4 py-2 font-medium">Most Common</th>
              <th class="px-4 py-2 font-medium">Repos Using</th>
              <th class="px-4 py-2 font-medium">Divergent Repos</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in driftRows"
              :key="row.configKey"
              class="border-b border-border last:border-b-0 hover:bg-surface/50"
            >
              <td class="px-4 py-2 font-medium">{{ row.configKey }}</td>
              <td class="px-4 py-2 font-mono text-xs text-success">{{ row.mostCommonValue }}</td>
              <td class="px-4 py-2 text-xs text-muted">{{ row.count }} / {{ row.total }}</td>
              <td class="px-4 py-2">
                <div v-if="row.divergentRepos.length === 0" class="text-xs text-success">
                  None
                </div>
                <div v-else class="space-y-1">
                  <div
                    v-for="dr in row.divergentRepos"
                    :key="dr.repoId"
                    class="flex items-center gap-2"
                  >
                    <span class="font-mono text-xs text-white">{{ dr.repoId }}</span>
                    <span class="font-mono text-xs text-warning">{{ dr.value }}</span>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>
