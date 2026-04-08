<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}>()
</script>

<template>
  <button
    :disabled="disabled || loading"
    class="inline-flex items-center justify-center gap-2 font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    :class="{
      'px-2.5 py-1 text-xs': size === 'sm',
      'px-3.5 py-1.5 text-sm': !size || size === 'md',
      'px-5 py-2 text-base': size === 'lg',
      'bg-primary text-white hover:bg-primary/80': !variant || variant === 'primary',
      'bg-surface-alt text-gray-200 border border-border hover:bg-white/10': variant === 'secondary',
      'bg-danger text-white hover:bg-danger/80': variant === 'danger',
      'text-muted hover:text-gray-200 hover:bg-white/5': variant === 'ghost',
    }"
  >
    <slot />
  </button>
</template>
