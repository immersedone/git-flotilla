<script setup lang="ts">
import { useRoute } from 'vue-router'
const route = useRoute()
</script>
<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6">CVE Incident: {{ route.params.id }}</h1>
    <p class="text-muted">Incident timeline — coming in Phase 5.</p>
  </div>
</template>
