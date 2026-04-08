<script setup lang="ts" generic="T">
defineProps<{
  items: T[]
  loading?: boolean
  emptyMessage?: string
}>()
</script>

<template>
  <div class="w-full overflow-x-auto">
    <table class="w-full text-sm border-collapse">
      <thead>
        <tr class="border-b border-border">
          <slot name="head" />
        </tr>
      </thead>
      <tbody>
        <tr v-if="loading">
          <td colspan="100%" class="py-12 text-center text-muted">Loading…</td>
        </tr>
        <tr v-else-if="items.length === 0">
          <td colspan="100%" class="py-12 text-center text-muted">
            {{ emptyMessage ?? 'No data' }}
          </td>
        </tr>
        <template v-else>
          <slot name="row" v-for="item in items" :item="item" />
        </template>
      </tbody>
    </table>
  </div>
</template>
