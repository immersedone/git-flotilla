import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Repo } from '@/types/repo'

export const useReposStore = defineStore('repos', () => {
  const repos = ref<Repo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const searchQuery = ref('')

  const filteredRepos = computed(() => {
    if (!searchQuery.value) return repos.value
    const q = searchQuery.value.toLowerCase()
    return repos.value.filter(r =>
      r.fullName.toLowerCase().includes(q) ||
      r.tags.some(t => t.toLowerCase().includes(q)),
    )
  })

  return { repos, isLoading, error, searchQuery, filteredRepos }
})
