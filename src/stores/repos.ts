import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Repo } from '@/types/repo'
import { listRepos, discoverRepos, setRepoTags } from '@/services/repos'

export const useReposStore = defineStore('repos', () => {
  const repos       = ref<Repo[]>([])
  const isLoading   = ref(false)
  const discovering = ref(false)
  const error       = ref<string | null>(null)
  const searchQuery = ref('')

  const filteredRepos = computed(() => {
    if (!searchQuery.value) return repos.value
    const q = searchQuery.value.toLowerCase()
    return repos.value.filter(r =>
      r.fullName.toLowerCase().includes(q) ||
      r.tags.some(t => t.toLowerCase().includes(q)),
    )
  })

  async function loadRepos(repoListId?: string) {
    isLoading.value = true
    error.value = null
    try {
      repos.value = await listRepos(repoListId)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function discoverReposAction(accountId: string) {
    discovering.value = true
    error.value = null
    try {
      repos.value = await discoverRepos(accountId)
    } catch (e) {
      error.value = String(e)
    } finally {
      discovering.value = false
    }
  }

  async function setRepoTagsAction(repoId: string, tags: string[]) {
    const updated = await setRepoTags(repoId, tags)
    const idx = repos.value.findIndex(r => r.id === repoId)
    if (idx !== -1) repos.value[idx] = updated
    return updated
  }

  return {
    repos, isLoading, discovering, error, searchQuery, filteredRepos,
    loadRepos, discoverReposAction, setRepoTagsAction,
  }
})
