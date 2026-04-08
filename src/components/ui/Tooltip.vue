<script setup lang="ts">
defineProps<{
  content: string
  position?: 'top' | 'bottom' | 'left' | 'right'
}>()
</script>

<template>
  <div class="relative inline-flex group">
    <slot />
    <div
      class="absolute z-50 px-2 py-1 text-xs text-white bg-gray-800 rounded whitespace-nowrap pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity"
      :class="{
        'bottom-full left-1/2 -translate-x-1/2 mb-1': !position || position === 'top',
        'top-full left-1/2 -translate-x-1/2 mt-1': position === 'bottom',
        'right-full top-1/2 -translate-y-1/2 mr-1': position === 'left',
        'left-full top-1/2 -translate-y-1/2 ml-1': position === 'right',
      }"
    >
      {{ content }}
    </div>
  </div>
</template>
