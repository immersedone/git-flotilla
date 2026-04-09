<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { Search, ArrowRight, Clock } from 'lucide-vue-next'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const router = useRouter()
const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const RECENT_KEY = 'flotilla-recent-commands'
const MAX_RECENT = 5

interface CommandItem {
  id: string
  label: string
  category: 'navigation' | 'action'
  to?: string
  action?: () => void
}

const allCommands: CommandItem[] = [
  { id: 'nav-dashboard',   label: 'Dashboard',       category: 'navigation', to: '/dashboard' },
  { id: 'nav-repos',       label: 'Repositories',    category: 'navigation', to: '/repos' },
  { id: 'nav-scanner',     label: 'Scanner',         category: 'navigation', to: '/scan' },
  { id: 'nav-packages',    label: 'Packages',        category: 'navigation', to: '/packages' },
  { id: 'nav-cve',         label: 'CVE Alerts',      category: 'navigation', to: '/cve' },
  { id: 'nav-ops',         label: 'Operations',      category: 'navigation', to: '/ops' },
  { id: 'nav-merge-queue', label: 'PR Merge Queue',  category: 'navigation', to: '/merge-queue' },
  { id: 'nav-scripts',     label: 'Script Runner',   category: 'navigation', to: '/scripts' },
  { id: 'nav-drift',       label: 'Drift Dashboard', category: 'navigation', to: '/drift' },
  { id: 'nav-compliance',  label: 'Compliance',      category: 'navigation', to: '/compliance' },
  { id: 'nav-settings',    label: 'Settings',        category: 'navigation', to: '/settings' },
  { id: 'nav-auth',        label: 'Accounts',        category: 'navigation', to: '/auth' },
  { id: 'act-scan-all',    label: 'Scan all repos',  category: 'action',     to: '/scan' },
  { id: 'act-check-cves',  label: 'Check CVEs',      category: 'action',     to: '/cve' },
  { id: 'act-export-csv',  label: 'Export CSV',       category: 'action',     to: '/packages' },
]

function getRecentIds(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY)
    if (!raw) return []
    const parsed: unknown = JSON.parse(raw)
    if (Array.isArray(parsed) && parsed.every((x): x is string => typeof x === 'string')) {
      return parsed.slice(0, MAX_RECENT)
    }
    return []
  } catch {
    return []
  }
}

function saveRecent(id: string) {
  const recent = getRecentIds().filter(r => r !== id)
  recent.unshift(id)
  localStorage.setItem(RECENT_KEY, JSON.stringify(recent.slice(0, MAX_RECENT)))
}

function fuzzyMatch(text: string, pattern: string): boolean {
  const lower = text.toLowerCase()
  const p = pattern.toLowerCase()
  let pi = 0
  for (let i = 0; i < lower.length && pi < p.length; i++) {
    if (lower[i] === p[pi]) pi++
  }
  return pi === p.length
}

const filteredCommands = computed(() => {
  const q = query.value.trim()
  const recentIds = getRecentIds()

  let items: CommandItem[]
  if (q === '') {
    // Show recent first, then remaining
    const recent = recentIds
      .map(id => allCommands.find(c => c.id === id))
      .filter((c): c is CommandItem => c !== undefined)
    const rest = allCommands.filter(c => !recentIds.includes(c.id))
    items = [...recent, ...rest]
  } else {
    items = allCommands.filter(c => fuzzyMatch(c.label, q))
  }
  return items
})

function isRecent(id: string): boolean {
  return query.value.trim() === '' && getRecentIds().includes(id)
}

function execute(item: CommandItem) {
  saveRecent(item.id)
  if (item.to) {
    router.push(item.to)
  }
  if (item.action) {
    item.action()
  }
  emit('update:open', false)
  query.value = ''
}

function handleKeydown(e: KeyboardEvent) {
  const len = filteredCommands.value.length
  if (len === 0) return

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = (selectedIndex.value + 1) % len
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = (selectedIndex.value - 1 + len) % len
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = filteredCommands.value[selectedIndex.value]
    if (item) execute(item)
  }
}

watch(() => props.open, async (open) => {
  if (!open) {
    query.value = ''
    selectedIndex.value = 0
  } else {
    await nextTick()
    inputRef.value?.focus()
  }
})

watch(query, () => {
  selectedIndex.value = 0
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-start justify-center pt-24 px-4"
    >
      <div class="absolute inset-0 bg-black/60" @click="emit('update:open', false)" />

      <div
        class="relative w-full max-w-xl bg-surface-alt border border-border rounded-xl shadow-2xl overflow-hidden"
        @keydown="handleKeydown"
      >
        <!-- Search input -->
        <div class="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Search class="w-4 h-4 text-muted shrink-0" />
          <input
            ref="inputRef"
            v-model="query"
            placeholder="Search repos, actions, views…"
            class="flex-1 bg-transparent text-sm text-gray-100 placeholder:text-muted outline-none"
          />
          <kbd class="text-xs text-muted bg-surface px-1.5 py-0.5 rounded border border-border">Esc</kbd>
        </div>

        <!-- Results -->
        <div class="max-h-80 overflow-y-auto py-2">
          <div
            v-if="filteredCommands.length === 0"
            class="px-4 py-6 text-center text-sm text-muted"
          >
            No results found
          </div>
          <button
            v-for="(item, idx) in filteredCommands"
            :key="item.id"
            class="w-full flex items-center px-4 py-2.5 text-sm text-gray-200 transition-colors text-left gap-3"
            :class="idx === selectedIndex ? 'bg-primary/15 text-primary' : 'hover:bg-white/5'"
            @click="execute(item)"
            @mouseenter="selectedIndex = idx"
          >
            <Clock v-if="isRecent(item.id)" class="w-3.5 h-3.5 text-muted shrink-0" />
            <ArrowRight v-else class="w-3.5 h-3.5 text-muted shrink-0" />
            <span class="flex-1">{{ item.label }}</span>
            <span class="text-xs text-muted font-mono">{{ item.category }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
