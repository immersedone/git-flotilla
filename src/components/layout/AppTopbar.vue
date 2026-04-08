<script setup lang="ts">
import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { Search, Bell, Zap } from 'lucide-vue-next'

const settingsStore = useSettingsStore()
const authStore = useAuthStore()

const emit = defineEmits<{
  'search': []
}>()

const rateLimitDisplay = computed(() => {
  const rl = settingsStore.rateLimitGithub
  if (!rl) return null
  const pct = Math.round((rl.remaining / rl.limit) * 100)
  const colour = pct < 20 ? 'text-danger' : pct < 50 ? 'text-warning' : 'text-success'
  return { text: rl.remaining.toLocaleString(), colour }
})
</script>

<template>
  <header class="h-12 bg-surface-alt border-b border-border flex items-center px-4 gap-4 flex-shrink-0">
    <!-- Left: Logo text (compact) -->
    <span class="text-muted text-xs font-mono w-44 flex-shrink-0">git-flotilla</span>

    <!-- Centre: Search trigger -->
    <button
      class="flex-1 flex items-center gap-2 bg-surface border border-border rounded-md px-3 py-1.5 text-sm text-muted hover:border-primary/50 transition-colors max-w-md"
      @click="emit('search')"
    >
      <Search class="w-3.5 h-3.5" />
      <span>Search repos, actions…</span>
      <kbd class="ml-auto text-xs bg-surface-alt px-1.5 py-0.5 rounded border border-border">⌘K</kbd>
    </button>

    <!-- Right: Rate limit, notif, auth -->
    <div class="flex items-center gap-3 ml-auto">
      <!-- GitHub API rate limit -->
      <div v-if="rateLimitDisplay" class="flex items-center gap-1.5 text-xs">
        <Zap class="w-3.5 h-3.5 text-muted" />
        <span :class="rateLimitDisplay.colour" class="font-mono">
          {{ rateLimitDisplay.text }}
        </span>
      </div>

      <!-- Notifications -->
      <button class="text-muted hover:text-gray-200 transition-colors">
        <Bell class="w-4 h-4" />
      </button>

      <!-- Auth status -->
      <div class="flex items-center gap-1.5 text-xs text-muted">
        <div
          class="w-2 h-2 rounded-full"
          :class="authStore.hasAccounts ? 'bg-success' : 'bg-danger'"
        />
        <span>{{ authStore.githubAccount?.username ?? 'Not connected' }}</span>
      </div>
    </div>
  </header>
</template>
