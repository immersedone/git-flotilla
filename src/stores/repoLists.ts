import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { RepoList } from '@/types/repo'

export const useRepoListsStore = defineStore('repoLists', () => {
  const lists = ref<RepoList[]>([])
  const selectedListId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const selectedList = computed(() =>
    lists.value.find(l => l.id === selectedListId.value) ?? null,
  )

  const rootLists = computed(() => lists.value.filter(l => l.parentId === null))

  return { lists, selectedListId, selectedList, rootLists, isLoading, error }
})
