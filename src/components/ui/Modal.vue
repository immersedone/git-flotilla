<script setup lang="ts">
defineProps<{
  open: boolean
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
}>()

defineEmits<{
  'update:open': [value: boolean]
}>()
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
    >
      <!-- Backdrop -->
      <div
        class="absolute inset-0 bg-black/60 backdrop-blur-sm"
        @click="$emit('update:open', false)"
      />

      <!-- Panel -->
      <div
        class="relative bg-surface-alt border border-border rounded-xl shadow-2xl w-full"
        :class="{
          'max-w-sm': size === 'sm',
          'max-w-lg': !size || size === 'md',
          'max-w-2xl': size === 'lg',
          'max-w-4xl': size === 'xl',
        }"
      >
        <div v-if="title" class="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 class="text-base font-semibold">{{ title }}</h2>
          <button
            class="text-muted hover:text-gray-200 transition-colors"
            @click="$emit('update:open', false)"
          >✕</button>
        </div>
        <div class="p-6">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>
