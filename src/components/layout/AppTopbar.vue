<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { Search, Bell, Zap, Check, Trash2 } from 'lucide-vue-next'

const settingsStore = useSettingsStore()
const authStore = useAuthStore()

const emit = defineEmits<{
  'search': []
}>()

const showNotifications = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

function toggleNotifications() {
  showNotifications.value = !showNotifications.value
  if (showNotifications.value) {
    settingsStore.loadNotifications()
  }
}

function handleClickOutside(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    showNotifications.value = false
  }
}

onMounted(() => document.addEventListener('click', handleClickOutside, true))
onUnmounted(() => document.removeEventListener('click', handleClickOutside, true))

function formatTime(ts: string): string {
  const d = new Date(ts)
  if (isNaN(d.getTime())) return ts
  const now = Date.now()
  const diff = now - d.getTime()
  if (diff < 60_000) return 'just now'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`
  return d.toLocaleDateString()
}

const rateLimitDisplay = computed(() => {
  const rl = settingsStore.rateLimitGithub
  if (!rl) return null
  const pct = Math.round((rl.remaining / rl.limit) * 100)
  const colour = pct < 20 ? 'text-danger' : pct < 50 ? 'text-warning' : 'text-success'
  return { text: rl.remaining.toLocaleString(), colour }
})
</script>

<template>
  <header class="h-12 bg-surface-alt border-b border-border flex items-center px-4 gap-4 shrink-0">
    <!-- Left: Logo text (compact) -->
    <span class="text-muted text-xs font-mono w-44 shrink-0">git-flotilla</span>

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
      <div ref="dropdownRef" class="relative">
        <button
          class="text-muted hover:text-gray-200 transition-colors relative"
          @click="toggleNotifications"
        >
          <Bell class="w-4 h-4" />
          <span
            v-if="settingsStore.unreadCount > 0"
            class="absolute -top-1.5 -right-1.5 bg-danger text-white text-[10px] font-bold rounded-full w-4 h-4 flex items-center justify-center"
          >
            {{ settingsStore.unreadCount > 9 ? '9+' : settingsStore.unreadCount }}
          </span>
        </button>

        <!-- Notification dropdown -->
        <div
          v-if="showNotifications"
          class="absolute right-0 top-8 w-80 bg-surface-alt border border-border rounded-lg shadow-2xl z-50 overflow-hidden"
        >
          <div class="flex items-center justify-between px-3 py-2 border-b border-border">
            <span class="text-sm font-semibold text-gray-200">Notifications</span>
            <button
              v-if="settingsStore.notifications.length > 0"
              class="text-xs text-muted hover:text-gray-200 flex items-center gap-1"
              @click="settingsStore.clearAllNotifications()"
            >
              <Trash2 class="w-3 h-3" />
              Clear all
            </button>
          </div>
          <div class="max-h-72 overflow-y-auto">
            <div
              v-if="settingsStore.notifications.length === 0"
              class="px-4 py-6 text-center text-sm text-muted"
            >
              No notifications
            </div>
            <button
              v-for="notif in settingsStore.notifications"
              :key="notif.id"
              class="w-full text-left px-3 py-2.5 border-b border-border last:border-b-0 hover:bg-white/5 transition-colors flex gap-2"
              :class="{ 'opacity-60': notif.isRead }"
              @click="settingsStore.markRead(notif.id)"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span
                    v-if="!notif.isRead"
                    class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"
                  />
                  <span class="text-sm text-gray-200 font-medium truncate">{{ notif.title }}</span>
                </div>
                <p class="text-xs text-muted mt-0.5 truncate">{{ notif.message }}</p>
                <span class="text-[10px] text-muted/70 mt-0.5 block">{{ formatTime(notif.timestamp) }}</span>
              </div>
              <Check v-if="notif.isRead" class="w-3 h-3 text-success shrink-0 mt-1" />
            </button>
          </div>
        </div>
      </div>

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
