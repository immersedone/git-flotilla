<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { IncidentTimeline } from '@/types/cve'
import { getCveIncident } from '@/services/cve'

const route = useRoute()
const router = useRouter()

const timeline = ref<IncidentTimeline | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)

const cveId = route.params.id as string

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-AU', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function eventTypeBadgeClass(eventType: string): string {
  switch (eventType) {
    case 'published':
      return 'bg-red-500/10 text-red-500 border-red-500/30'
    case 'detected':
      return 'bg-amber-500/10 text-amber-500 border-amber-500/30'
    case 'pr_created':
    case 'pr_merged':
      return 'bg-green-500/10 text-green-500 border-green-500/30'
    case 'acknowledged':
      return 'bg-blue-500/10 text-blue-500 border-blue-500/30'
    case 'dismissed':
    case 'snoozed':
      return 'bg-gray-500/10 text-gray-400 border-gray-500/30'
    default:
      return 'bg-gray-500/10 text-gray-400 border-gray-500/30'
  }
}

async function loadTimeline() {
  isLoading.value = true
  error.value = null
  try {
    timeline.value = await getCveIncident(cveId)
  } catch (e) {
    error.value = String(e)
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  loadTimeline()
})
</script>

<template>
  <div class="space-y-6">
    <!-- Back link + header -->
    <div>
      <button
        class="text-sm text-gray-500 hover:text-gray-300 mb-2"
        @click="router.push('/cve')"
      >
        &larr; Back to CVE Alerts
      </button>
      <h1 class="text-2xl font-semibold text-white">
        Incident: <span class="font-mono">{{ cveId }}</span>
      </h1>
    </div>

    <!-- Error display -->
    <div
      v-if="error"
      class="bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-400 text-sm"
    >
      {{ error }}
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="text-gray-500 text-sm">Loading incident timeline...</div>

    <!-- Timeline content -->
    <template v-if="timeline && !isLoading">
      <!-- Metadata -->
      <div class="grid grid-cols-3 gap-4">
        <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
          <div class="text-sm text-gray-500">CVE ID</div>
          <div class="text-lg font-mono text-white mt-1">{{ timeline.cveId }}</div>
        </div>
        <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
          <div class="text-sm text-gray-500">Published</div>
          <div class="text-sm text-white mt-1">{{ formatDate(timeline.publishedAt) }}</div>
        </div>
        <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-4">
          <div class="text-sm text-gray-500">Detected by Flotilla</div>
          <div class="text-sm text-white mt-1">{{ formatDate(timeline.detectedAt) }}</div>
        </div>
      </div>

      <!-- Timeline events -->
      <div class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-6">
        <h2 class="text-lg font-semibold text-white mb-4">Timeline</h2>

        <div v-if="timeline.events.length === 0" class="text-sm text-gray-600">
          No events recorded yet.
        </div>

        <div v-else class="relative">
          <!-- Vertical line -->
          <div class="absolute left-3 top-2 bottom-2 w-px bg-[#2A2D3A]" />

          <div
            v-for="(event, idx) in timeline.events"
            :key="idx"
            class="relative pl-10 pb-6 last:pb-0"
          >
            <!-- Dot -->
            <div class="absolute left-1.5 top-1.5 w-3 h-3 rounded-full bg-[#2A2D3A] border-2 border-[#1A1D27]" />

            <div class="flex items-start gap-3">
              <span
                class="inline-block px-2 py-0.5 rounded border text-xs font-medium shrink-0"
                :class="eventTypeBadgeClass(event.eventType)"
              >
                {{ event.eventType }}
              </span>
              <span class="text-xs text-gray-500 shrink-0 pt-0.5">
                {{ formatDate(event.timestamp) }}
              </span>
            </div>
            <p class="text-sm text-gray-300 mt-1">{{ event.detail }}</p>
            <span
              v-if="event.repoId"
              class="inline-block mt-1 px-2 py-0.5 bg-[#0F1117] border border-[#2A2D3A] rounded text-xs font-mono text-gray-400"
            >
              {{ event.repoId }}
            </span>
          </div>
        </div>
      </div>
    </template>

    <!-- Empty state when no timeline loaded and not loading -->
    <div
      v-if="!timeline && !isLoading && !error"
      class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg p-12 text-center"
    >
      <div class="text-gray-500 text-lg mb-2">No incident data</div>
      <p class="text-gray-600 text-sm">Could not find incident timeline for this CVE.</p>
    </div>
  </div>
</template>
