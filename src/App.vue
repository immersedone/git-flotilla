<script setup lang="ts">
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppTopbar from '@/components/layout/AppTopbar.vue'
import CommandPalette from '@/components/ui/CommandPalette.vue'
import { useSettingsStore } from '@/stores/settings'
import { ref, onMounted, onUnmounted } from 'vue'

const settingsStore = useSettingsStore()
const commandPaletteOpen = ref(false)

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    commandPaletteOpen.value = !commandPaletteOpen.value
  }
  if (e.key === 'Escape') {
    commandPaletteOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  settingsStore.initTheme()
})
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <div class="flex flex-col h-full bg-surface text-gray-100">
    <AppTopbar @search="commandPaletteOpen = true" />
    <div class="flex flex-1 overflow-hidden">
      <AppSidebar />
      <main class="flex-1 overflow-auto p-6">
        <RouterView />
      </main>
    </div>
    <CommandPalette v-model:open="commandPaletteOpen" />
  </div>
</template>

<style>
html.light {
  --color-surface: #ffffff;
  --color-surface-alt: #f8fafc;
  --color-border: #e2e8f0;
  --color-text: #1e293b;
  --color-muted: #64748b;
}
</style>
