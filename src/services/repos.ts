import { invoke } from '@tauri-apps/api/core'
import type { Repo, RepoList } from '@/types/repo'

export interface CreateRepoListInput {
  name: string
  description: string
  parentId?: string
}

export function discoverRepos(accountId: string): Promise<Repo[]> {
  return invoke('discover_repos', { accountId })
}

export function listRepos(repoListId?: string): Promise<Repo[]> {
  return invoke('list_repos', { repoListId: repoListId ?? null })
}

export function getRepo(id: string): Promise<Repo> {
  return invoke('get_repo', { id })
}

export function createRepoList(input: CreateRepoListInput): Promise<RepoList> {
  return invoke('create_repo_list', { input })
}

export function updateRepoList(id: string, input: CreateRepoListInput): Promise<RepoList> {
  return invoke('update_repo_list', { id, input })
}

export function deleteRepoList(id: string): Promise<void> {
  return invoke('delete_repo_list', { id })
}

export function listRepoLists(): Promise<RepoList[]> {
  return invoke('list_repo_lists')
}

export function addReposToList(listId: string, repoIds: string[]): Promise<void> {
  return invoke('add_repos_to_list', { listId, repoIds })
}

export function removeReposFromList(listId: string, repoIds: string[]): Promise<void> {
  return invoke('remove_repos_from_list', { listId, repoIds })
}
