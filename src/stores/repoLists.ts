import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { RepoList } from '@/types/repo'
import type { CreateRepoListInput } from '@/services/repos'
import {
  listRepoLists, createRepoList, updateRepoList, deleteRepoList,
  addReposToList, removeReposFromList, exportRepoList, importRepoList,
} from '@/services/repos'

export const useRepoListsStore = defineStore('repoLists', () => {
  const lists          = ref<RepoList[]>([])
  const selectedListId = ref<string | null>(null)
  const isLoading      = ref(false)
  const error          = ref<string | null>(null)

  const selectedList = computed(() =>
    lists.value.find(l => l.id === selectedListId.value) ?? null,
  )
  const rootLists = computed(() => lists.value.filter(l => l.parentId === null))

  async function loadLists() {
    isLoading.value = true
    error.value = null
    try {
      lists.value = await listRepoLists()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createListAction(input: CreateRepoListInput) {
    const list = await createRepoList(input)
    lists.value.push(list)
    return list
  }

  async function updateListAction(id: string, input: CreateRepoListInput) {
    const updated = await updateRepoList(id, input)
    const idx = lists.value.findIndex(l => l.id === id)
    if (idx !== -1) lists.value[idx] = updated
    return updated
  }

  async function deleteListAction(id: string) {
    await deleteRepoList(id)
    lists.value = lists.value.filter(l => l.id !== id)
    if (selectedListId.value === id) selectedListId.value = null
  }

  async function addRepos(listId: string, repoIds: string[]) {
    await addReposToList(listId, repoIds)
    await loadLists()
  }

  async function removeRepos(listId: string, repoIds: string[]) {
    await removeReposFromList(listId, repoIds)
    await loadLists()
  }

  async function exportList(id: string): Promise<string> {
    return exportRepoList(id)
  }

  async function importList(yaml: string) {
    const list = await importRepoList(yaml)
    const idx = lists.value.findIndex(l => l.id === list.id)
    if (idx !== -1) {
      lists.value[idx] = list
    } else {
      lists.value.push(list)
    }
    return list
  }

  return {
    lists, selectedListId, selectedList, rootLists, isLoading, error,
    loadLists, createListAction, updateListAction, deleteListAction,
    addRepos, removeRepos, exportList, importList,
  }
})
