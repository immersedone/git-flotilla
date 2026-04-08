<script setup lang="ts">
defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  type?: string
  error?: string
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div class="flex flex-col gap-1">
    <input
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :type="type ?? 'text'"
      class="bg-surface border rounded-md px-3 py-1.5 text-sm text-gray-100 placeholder:text-muted outline-none transition-colors disabled:opacity-50"
      :class="error
        ? 'border-danger focus:border-danger'
        : 'border-border focus:border-primary'"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <p v-if="error" class="text-danger text-xs">{{ error }}</p>
  </div>
</template>
