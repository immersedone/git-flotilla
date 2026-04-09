<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useMergeQueueStore } from '@/stores/mergeQueue'

const store = useMergeQueueStore()
const merging = ref<Record<string, boolean>>({})

onMounted(() => {
  store.loadPrs().catch(() => {})
})

async function handleMerge(repoId: string, prNumber: number) {
  const key = `${repoId}:${prNumber}`
  merging.value[key] = true
  try {
    await store.mergeSinglePr(repoId, prNumber)
  } finally {
    merging.value[key] = false
  }
}

async function handleMergeAllGreen() {
  await store.mergeAllGreenAction()
}

function statusColor(ciStatus: string | null): string {
  switch (ciStatus) {
    case 'success': return 'text-success'
    case 'failure': return 'text-danger'
    case 'pending': return 'text-warning'
    default: return 'text-muted'
  }
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  return d.toLocaleDateString()
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">PR Merge Queue</h1>
      <button
        v-if="store.greenPrs.length > 0"
        class="px-4 py-2 rounded-md bg-success/20 text-success text-sm font-medium hover:bg-success/30 transition-colors"
        :disabled="store.isLoading"
        @click="handleMergeAllGreen"
      >
        Merge All Green ({{ store.greenPrs.length }})
      </button>
    </div>

    <p v-if="store.error" class="text-danger text-sm">{{ store.error }}</p>

    <!-- Loading -->
    <div v-if="store.isLoading && store.prs.length === 0" class="text-muted text-sm">
      Loading PRs...
    </div>

    <!-- Empty state -->
    <div
      v-else-if="store.prs.length === 0"
      class="rounded-lg bg-surface-alt border border-border p-8 text-center text-muted"
    >
      <p class="text-lg font-medium mb-1">No open Flotilla PRs</p>
      <p class="text-sm">PRs created by batch operations will appear here.</p>
    </div>

    <!-- PR table -->
    <div v-else class="rounded-lg bg-surface-alt border border-border overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border text-left text-muted">
            <th class="px-4 py-2 font-medium">Repository</th>
            <th class="px-4 py-2 font-medium">PR</th>
            <th class="px-4 py-2 font-medium">Title</th>
            <th class="px-4 py-2 font-medium">CI Status</th>
            <th class="px-4 py-2 font-medium">Mergeable</th>
            <th class="px-4 py-2 font-medium">Created</th>
            <th class="px-4 py-2 font-medium text-right">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="pr in store.prs"
            :key="`${pr.repoId}:${pr.prNumber}`"
            class="border-b border-border last:border-b-0 hover:bg-surface/50"
          >
            <td class="px-4 py-2 font-mono text-xs">{{ pr.repoId }}</td>
            <td class="px-4 py-2">
              <a
                :href="pr.htmlUrl"
                target="_blank"
                rel="noopener"
                class="text-primary hover:underline font-mono text-xs"
              >
                #{{ pr.prNumber }}
              </a>
            </td>
            <td class="px-4 py-2 truncate max-w-xs">{{ pr.title }}</td>
            <td class="px-4 py-2">
              <span class="font-mono text-xs" :class="statusColor(pr.ciStatus)">
                {{ pr.ciStatus ?? 'unknown' }}
              </span>
            </td>
            <td class="px-4 py-2">
              <span
                class="inline-block rounded px-2 py-0.5 text-xs font-mono"
                :class="{
                  'bg-success/20 text-success': pr.mergeable === 'MERGEABLE',
                  'bg-danger/20 text-danger': pr.mergeable === 'CONFLICTING',
                  'bg-surface text-muted border border-border': pr.mergeable !== 'MERGEABLE' && pr.mergeable !== 'CONFLICTING',
                }"
              >
                {{ pr.mergeable ?? 'unknown' }}
              </span>
            </td>
            <td class="px-4 py-2 text-xs text-muted whitespace-nowrap">
              {{ formatDate(pr.createdAt) }}
            </td>
            <td class="px-4 py-2 text-right">
              <button
                class="px-3 py-1 rounded text-xs font-medium transition-colors"
                :class="pr.ciStatus === 'success' && pr.mergeable === 'MERGEABLE'
                  ? 'bg-primary/20 text-primary hover:bg-primary/30'
                  : 'bg-surface text-muted border border-border cursor-not-allowed opacity-50'"
                :disabled="pr.ciStatus !== 'success' || pr.mergeable !== 'MERGEABLE' || merging[`${pr.repoId}:${pr.prNumber}`]"
                @click="handleMerge(pr.repoId, pr.prNumber)"
              >
                {{ merging[`${pr.repoId}:${pr.prNumber}`] ? 'Merging...' : 'Merge' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
