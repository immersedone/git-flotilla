<script setup lang="ts">
defineProps<{
  padding?: boolean
}>()
</script>

<template>
  <div
    class="bg-surface-alt border border-border rounded-lg"
    :class="{ 'p-4': padding !== false }"
  >
    <slot />
  </div>
</template>
