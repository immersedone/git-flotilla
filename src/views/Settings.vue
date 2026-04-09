<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useSettingsStore } from '@/stores/settings'

const store = useSettingsStore()
const saving = ref(false)

const form = ref({
  scan_interval: '1440',
  inter_request_delay_ms: '200',
  divergence_threshold: '50',
  cve_poll_interval: '60',
  parallel_workers: '5',
})

onMounted(async () => {
  try {
    await Promise.all([
      store.loadSettings(),
      store.refreshRateLimit(),
      store.loadAuditLog(20),
    ])
  } catch {
    // errors captured in store
  }
  // Populate form from loaded settings
  if (store.settings.scan_interval) form.value.scan_interval = store.settings.scan_interval
  if (store.settings.inter_request_delay_ms) form.value.inter_request_delay_ms = store.settings.inter_request_delay_ms
  if (store.settings.divergence_threshold) form.value.divergence_threshold = store.settings.divergence_threshold
  if (store.settings.cve_poll_interval) form.value.cve_poll_interval = store.settings.cve_poll_interval
  if (store.settings.parallel_workers) form.value.parallel_workers = store.settings.parallel_workers
})

async function handleSave() {
  saving.value = true
  try {
    await store.saveSettingsAction({ ...form.value })
  } finally {
    saving.value = false
  }
}

function formatTimestamp(ts: unknown): string {
  if (typeof ts !== 'string') return '-'
  const d = new Date(ts)
  if (isNaN(d.getTime())) return String(ts)
  return d.toLocaleString()
}
</script>

<template>
  <div class="space-y-8">
    <h1 class="text-2xl font-semibold">Settings</h1>

    <p v-if="store.error" class="text-danger text-sm">{{ store.error }}</p>

    <!-- Theme Toggle -->
    <section class="rounded-lg bg-surface-alt border border-border p-6 space-y-3">
      <h2 class="text-lg font-semibold">Appearance</h2>
      <div class="flex items-center gap-4">
        <span class="text-sm text-muted">Theme</span>
        <button
          class="px-4 py-2 rounded-md text-sm font-medium border transition-colors"
          :class="store.theme === 'dark'
            ? 'bg-primary/20 text-primary border-primary/50'
            : 'bg-surface border-border text-muted hover:text-gray-200'"
          @click="store.theme !== 'dark' && store.toggleTheme()"
        >
          Dark
        </button>
        <button
          class="px-4 py-2 rounded-md text-sm font-medium border transition-colors"
          :class="store.theme === 'light'
            ? 'bg-primary/20 text-primary border-primary/50'
            : 'bg-surface border-border text-muted hover:text-gray-200'"
          @click="store.theme !== 'light' && store.toggleTheme()"
        >
          Light
        </button>
      </div>
    </section>

    <!-- Configuration -->
    <section class="rounded-lg bg-surface-alt border border-border p-6 space-y-5">
      <h2 class="text-lg font-semibold">Configuration</h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <div>
          <label class="block text-sm text-muted mb-1">Scan Interval (minutes)</label>
          <input
            v-model="form.scan_interval"
            type="number"
            min="1"
            class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
          />
        </div>

        <div>
          <label class="block text-sm text-muted mb-1">CVE Poll Interval (minutes)</label>
          <input
            v-model="form.cve_poll_interval"
            type="number"
            min="0"
            class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
          />
        </div>

        <div>
          <label class="block text-sm text-muted mb-1">Inter-Request Delay (ms)</label>
          <input
            v-model="form.inter_request_delay_ms"
            type="number"
            min="0"
            class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
          />
        </div>

        <div>
          <label class="block text-sm text-muted mb-1">Parallel Workers</label>
          <input
            v-model="form.parallel_workers"
            type="number"
            min="1"
            max="20"
            class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
          />
        </div>

        <div>
          <label class="block text-sm text-muted mb-1">Divergence Threshold (commits)</label>
          <input
            v-model="form.divergence_threshold"
            type="number"
            min="1"
            class="w-full rounded-md bg-surface border border-border px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
          />
        </div>
      </div>

      <div class="pt-2">
        <button
          class="px-4 py-2 rounded-md bg-primary text-white text-sm font-medium hover:bg-primary/80 transition-colors disabled:opacity-50"
          :disabled="saving || store.isLoading"
          @click="handleSave"
        >
          {{ saving ? 'Saving...' : 'Save Settings' }}
        </button>
      </div>
    </section>

    <!-- Rate Limit -->
    <section class="rounded-lg bg-surface-alt border border-border p-6 space-y-3">
      <h2 class="text-lg font-semibold">API Rate Limits</h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div class="rounded-md bg-surface border border-border p-4">
          <p class="text-sm text-muted mb-1">GitHub</p>
          <template v-if="store.rateLimitGithub">
            <p class="text-2xl font-bold font-mono" :class="{
              'text-success': store.rateLimitGithub.remaining > 1000,
              'text-warning': store.rateLimitGithub.remaining <= 1000 && store.rateLimitGithub.remaining > 100,
              'text-danger': store.rateLimitGithub.remaining <= 100,
            }">
              {{ store.rateLimitGithub.remaining }}
              <span class="text-sm text-muted font-normal">/ {{ store.rateLimitGithub.limit }}</span>
            </p>
            <p class="text-xs text-muted mt-1">
              Resets {{ new Date(store.rateLimitGithub.resetEpoch * 1000).toLocaleTimeString() }}
            </p>
          </template>
          <p v-else class="text-sm text-muted">Not connected</p>
        </div>

        <div class="rounded-md bg-surface border border-border p-4">
          <p class="text-sm text-muted mb-1">GitLab</p>
          <template v-if="store.rateLimitGitlab">
            <p class="text-2xl font-bold font-mono" :class="{
              'text-success': store.rateLimitGitlab.remaining > 1000,
              'text-warning': store.rateLimitGitlab.remaining <= 1000 && store.rateLimitGitlab.remaining > 100,
              'text-danger': store.rateLimitGitlab.remaining <= 100,
            }">
              {{ store.rateLimitGitlab.remaining }}
              <span class="text-sm text-muted font-normal">/ {{ store.rateLimitGitlab.limit }}</span>
            </p>
            <p class="text-xs text-muted mt-1">
              Resets {{ new Date(store.rateLimitGitlab.resetEpoch * 1000).toLocaleTimeString() }}
            </p>
          </template>
          <p v-else class="text-sm text-muted">Not connected</p>
        </div>
      </div>
    </section>

    <!-- Audit Log -->
    <section class="rounded-lg bg-surface-alt border border-border p-6 space-y-3">
      <h2 class="text-lg font-semibold">Audit Log</h2>

      <div v-if="store.auditLog.length === 0" class="text-muted text-sm">
        No audit log entries
      </div>
      <div v-else class="overflow-hidden rounded-md border border-border">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted bg-surface">
              <th class="px-4 py-2 font-medium">Timestamp</th>
              <th class="px-4 py-2 font-medium">Action</th>
              <th class="px-4 py-2 font-medium">Details</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(entry, idx) in store.auditLog"
              :key="idx"
              class="border-b border-border last:border-b-0 hover:bg-surface/50"
            >
              <td class="px-4 py-2 font-mono text-xs whitespace-nowrap">
                {{ formatTimestamp(entry.timestamp) }}
              </td>
              <td class="px-4 py-2">
                <span class="inline-block rounded bg-surface px-2 py-0.5 text-xs font-mono border border-border">
                  {{ entry.action_type ?? entry.actionType ?? '-' }}
                </span>
              </td>
              <td class="px-4 py-2 text-muted truncate max-w-md">
                {{ entry.summary ?? entry.details ?? '-' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </div>
</template>
