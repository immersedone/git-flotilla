<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Search } from 'lucide-vue-next'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const router = useRouter()
const query = ref('')

const quickActions = [
  { label: 'Dashboard',     to: '/dashboard' },
  { label: 'Repositories',  to: '/repos' },
  { label: 'Scanner',       to: '/scan' },
  { label: 'CVE Alerts',    to: '/cve' },
  { label: 'Operations',    to: '/ops' },
  { label: 'PR Queue',      to: '/merge-queue' },
  { label: 'Script Runner', to: '/scripts' },
  { label: 'Compliance',    to: '/compliance' },
  { label: 'Settings',      to: '/settings' },
]

function navigate(to: string) {
  router.push(to)
  emit('update:open', false)
  query.value = ''
}

watch(() => props.open, (open) => {
  if (!open) query.value = ''
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-start justify-center pt-24 px-4"
    >
      <div class="absolute inset-0 bg-black/60" @click="emit('update:open', false)" />

      <div class="relative w-full max-w-xl bg-surface-alt border border-border rounded-xl shadow-2xl overflow-hidden">
        <!-- Search input -->
        <div class="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Search class="w-4 h-4 text-muted flex-shrink-0" />
          <input
            v-model="query"
            placeholder="Search repos, actions, views…"
            class="flex-1 bg-transparent text-sm text-gray-100 placeholder:text-muted outline-none"
            autofocus
          />
          <kbd class="text-xs text-muted bg-surface px-1.5 py-0.5 rounded border border-border">Esc</kbd>
        </div>

        <!-- Results -->
        <div class="max-h-80 overflow-y-auto py-2">
          <button
            v-for="action in quickActions"
            :key="action.to"
            class="w-full flex items-center px-4 py-2.5 text-sm text-gray-200 hover:bg-white/5 transition-colors text-left"
            @click="navigate(action.to)"
          >
            {{ action.label }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
