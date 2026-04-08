<script setup lang="ts">
defineProps<{
  variant: 'critical' | 'high' | 'medium' | 'low' | 'info' | 'success' | 'muted'
}>()
</script>

<template>
  <span
    class="inline-flex items-center px-2 py-0.5 text-xs font-semibold rounded-full uppercase tracking-wide"
    :class="{
      'bg-danger/20 text-danger': variant === 'critical',
      'bg-orange-500/20 text-orange-400': variant === 'high',
      'bg-warning/20 text-warning': variant === 'medium',
      'bg-blue-500/20 text-blue-400': variant === 'low',
      'bg-primary/20 text-primary': variant === 'info',
      'bg-success/20 text-success': variant === 'success',
      'bg-surface-alt text-muted': variant === 'muted',
    }"
  >
    <slot />
  </span>
</template>
