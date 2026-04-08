<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useReposStore } from '@/stores/repos'
import { useRepoListsStore } from '@/stores/repoLists'
import { useAuthStore } from '@/stores/auth'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Modal from '@/components/ui/Modal.vue'
import type { CreateRepoListInput } from '@/services/repos'

const reposStore     = useReposStore()
const listsStore     = useRepoListsStore()
const authStore      = useAuthStore()

const activeTab      = ref<'repos' | 'lists'>('repos')
const showCreateList = ref(false)
const showAddRepos   = ref(false)
const importYaml     = ref('')
const exportedYaml   = ref('')
const showExport     = ref(false)
const actionError    = ref<string | null>(null)

onMounted(async () => {
  await Promise.all([reposStore.loadRepos(), listsStore.loadLists()])
})

// ── Discover ─────────────────────────────────────────────────────────────
async function handleDiscover() {
  if (!authStore.githubAccount) return
  actionError.value = null
  try {
    await reposStore.discoverReposAction(authStore.githubAccount.id)
  } catch (e) {
    actionError.value = String(e)
  }
}

// ── Create list form ──────────────────────────────────────────────────────
const newListName        = ref('')
const newListDescription = ref('')

async function handleCreateList() {
  if (!newListName.value.trim()) return
  actionError.value = null
  try {
    const input: CreateRepoListInput = {
      name:             newListName.value.trim(),
      description:      newListDescription.value.trim(),
      excludePatterns:  [],
    }
    await listsStore.createListAction(input)
    newListName.value        = ''
    newListDescription.value = ''
    showCreateList.value     = false
  } catch (e) {
    actionError.value = String(e)
  }
}

async function handleDeleteList(id: string) {
  actionError.value = null
  try {
    await listsStore.deleteListAction(id)
  } catch (e) {
    actionError.value = String(e)
  }
}

// ── Add repos to selected list ────────────────────────────────────────────
const addRepoSearch  = ref('')
const selectedToAdd  = ref<Set<string>>(new Set())

const reposNotInList = computed(() => {
  const listRepoIds = new Set(listsStore.selectedList?.repoIds ?? [])
  const q = addRepoSearch.value.toLowerCase()
  return reposStore.repos.filter(r =>
    !listRepoIds.has(r.id) &&
    (!q || r.fullName.toLowerCase().includes(q)),
  )
})

function toggleRepoSelect(id: string) {
  if (selectedToAdd.value.has(id)) {
    selectedToAdd.value.delete(id)
  } else {
    selectedToAdd.value.add(id)
  }
}

async function handleAddRepos() {
  if (!listsStore.selectedListId || selectedToAdd.value.size === 0) return
  actionError.value = null
  try {
    await listsStore.addRepos(listsStore.selectedListId, [...selectedToAdd.value])
    selectedToAdd.value = new Set()
    showAddRepos.value  = false
  } catch (e) {
    actionError.value = String(e)
  }
}

async function handleRemoveFromList(repoId: string) {
  if (!listsStore.selectedListId) return
  actionError.value = null
  try {
    await listsStore.removeRepos(listsStore.selectedListId, [repoId])
  } catch (e) {
    actionError.value = String(e)
  }
}

// ── Export / Import ───────────────────────────────────────────────────────
async function handleExport(id: string) {
  actionError.value = null
  try {
    exportedYaml.value = await listsStore.exportList(id)
    showExport.value   = true
  } catch (e) {
    actionError.value = String(e)
  }
}

async function handleImport() {
  if (!importYaml.value.trim()) return
  actionError.value = null
  try {
    await listsStore.importList(importYaml.value.trim())
    importYaml.value = ''
  } catch (e) {
    actionError.value = String(e)
  }
}

async function selectList(id: string) {
  listsStore.selectedListId = id
  await reposStore.loadRepos(id)
}

const reposInSelectedList = computed(() => {
  if (!listsStore.selectedListId) return []
  const ids = new Set(listsStore.selectedList?.repoIds ?? [])
  return reposStore.repos.filter(r => ids.has(r.id))
})
</script>

<template>
  <div class="p-6 flex flex-col gap-6 h-full">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Repositories</h1>
      <div class="flex items-center gap-2">
        <!-- Tab toggles -->
        <div class="flex border border-border rounded-md overflow-hidden text-sm">
          <button
            class="px-4 py-1.5 transition-colors"
            :class="activeTab === 'repos' ? 'bg-primary text-white' : 'text-muted hover:text-gray-200'"
            @click="activeTab = 'repos'"
          >Repos</button>
          <button
            class="px-4 py-1.5 transition-colors border-l border-border"
            :class="activeTab === 'lists' ? 'bg-primary text-white' : 'text-muted hover:text-gray-200'"
            @click="activeTab = 'lists'"
          >Lists</button>
        </div>
      </div>
    </div>

    <p v-if="actionError" class="text-danger text-sm">{{ actionError }}</p>

    <!-- Tab: Repos ─────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'repos'" class="flex flex-col gap-4 flex-1 min-h-0">
      <div class="flex items-center gap-3">
        <Input
          v-model="reposStore.searchQuery"
          placeholder="Search repos or tags…"
          class="w-72"
        />
        <Button
          variant="primary"
          :loading="reposStore.discovering"
          :disabled="!authStore.hasAccounts || reposStore.discovering"
          @click="handleDiscover"
        >
          {{ reposStore.discovering ? 'Discovering…' : 'Discover Repos' }}
        </Button>
        <span v-if="!authStore.hasAccounts" class="text-xs text-warning">
          Connect an account in Accounts first
        </span>
      </div>

      <!-- Repos table -->
      <div class="flex-1 overflow-auto">
        <table class="w-full text-sm">
          <thead class="sticky top-0 bg-surface border-b border-border">
            <tr class="text-left text-muted">
              <th class="py-2 pr-4 font-medium pl-2">Repository</th>
              <th class="py-2 pr-4 font-medium">Branch</th>
              <th class="py-2 pr-4 font-medium">Tags</th>
              <th class="py-2 font-medium">Visibility</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="reposStore.isLoading">
              <td colspan="4" class="py-8 text-center text-muted">Loading repos…</td>
            </tr>
            <tr v-else-if="reposStore.filteredRepos.length === 0">
              <td colspan="4" class="py-8 text-center text-muted">
                No repos found. Click "Discover Repos" to scan your GitHub account.
              </td>
            </tr>
            <tr
              v-for="repo in reposStore.filteredRepos"
              :key="repo.id"
              class="border-b border-border/50 hover:bg-white/5 transition-colors"
            >
              <td class="py-2 pr-4 pl-2">
                <a :href="repo.url" target="_blank" class="text-primary hover:underline font-mono text-xs">
                  {{ repo.fullName }}
                </a>
              </td>
              <td class="py-2 pr-4 font-mono text-xs text-muted">{{ repo.defaultBranch }}</td>
              <td class="py-2 pr-4">
                <div class="flex flex-wrap gap-1">
                  <span
                    v-for="tag in repo.tags"
                    :key="tag"
                    class="text-xs bg-primary/10 text-primary px-1.5 py-0.5 rounded"
                  >{{ tag }}</span>
                </div>
              </td>
              <td class="py-2">
                <span class="text-xs" :class="repo.isPrivate ? 'text-warning' : 'text-muted'">
                  {{ repo.isPrivate ? 'Private' : 'Public' }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Tab: Lists ─────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'lists'" class="flex gap-4 flex-1 min-h-0">
      <!-- Left: list tree -->
      <aside class="w-56 shrink-0 flex flex-col gap-2">
        <Button variant="secondary" size="sm" class="w-full" @click="showCreateList = true">
          + New List
        </Button>

        <!-- Import YAML -->
        <div class="space-y-1">
          <textarea
            v-model="importYaml"
            rows="3"
            placeholder="Paste YAML to import…"
            class="w-full text-xs bg-surface border border-border rounded-md px-2 py-1.5 text-gray-100 placeholder:text-muted resize-none outline-none focus:border-primary"
          />
          <Button variant="ghost" size="sm" :disabled="!importYaml.trim()" @click="handleImport">
            Import
          </Button>
        </div>

        <div class="border-t border-border pt-2 flex flex-col gap-0.5">
          <div v-if="listsStore.isLoading" class="text-xs text-muted px-2">Loading…</div>
          <button
            v-for="list in listsStore.rootLists"
            :key="list.id"
            class="text-left px-2 py-1.5 rounded-md text-sm transition-colors truncate"
            :class="listsStore.selectedListId === list.id
              ? 'bg-primary/20 text-primary'
              : 'text-muted hover:text-gray-200 hover:bg-white/5'"
            @click="selectList(list.id)"
          >
            {{ list.name }}
            <span class="text-xs opacity-60 ml-1">({{ list.repoIds.length }})</span>
          </button>
          <p v-if="!listsStore.isLoading && listsStore.rootLists.length === 0" class="text-xs text-muted px-2">
            No lists yet
          </p>
        </div>
      </aside>

      <!-- Right: repos in selected list -->
      <main class="flex-1 flex flex-col gap-3 min-w-0 overflow-hidden">
        <div v-if="!listsStore.selectedList" class="text-muted text-sm pt-4">
          Select a list to manage its repos.
        </div>

        <template v-else>
          <div class="flex items-center justify-between shrink-0">
            <div>
              <h2 class="font-semibold">{{ listsStore.selectedList.name }}</h2>
              <p class="text-xs text-muted">{{ listsStore.selectedList.description }}</p>
            </div>
            <div class="flex gap-2">
              <Button variant="secondary" size="sm" @click="handleExport(listsStore.selectedListId!)">
                Export YAML
              </Button>
              <Button variant="secondary" size="sm" @click="showAddRepos = true">
                Add Repos
              </Button>
              <Button variant="danger" size="sm" @click="handleDeleteList(listsStore.selectedListId!)">
                Delete List
              </Button>
            </div>
          </div>

          <div class="flex-1 overflow-auto">
            <table class="w-full text-sm">
              <thead class="sticky top-0 bg-surface border-b border-border">
                <tr class="text-left text-muted">
                  <th class="py-2 pr-4 font-medium pl-2">Repository</th>
                  <th class="py-2 font-medium">Action</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="reposInSelectedList.length === 0">
                  <td colspan="2" class="py-6 text-center text-muted text-sm">
                    No repos in this list. Click "Add Repos" to add some.
                  </td>
                </tr>
                <tr
                  v-for="repo in reposInSelectedList"
                  :key="repo.id"
                  class="border-b border-border/50 hover:bg-white/5"
                >
                  <td class="py-2 pr-4 pl-2 font-mono text-xs">{{ repo.fullName }}</td>
                  <td class="py-2">
                    <Button variant="ghost" size="sm" @click="handleRemoveFromList(repo.id)">
                      Remove
                    </Button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </main>
    </div>

    <!-- Modal: Create List ──────────────────────────────────────────────── -->
    <Modal v-model:open="showCreateList" title="Create Repo List" size="sm">
      <div class="space-y-3">
        <div>
          <label class="block text-sm text-muted mb-1">Name</label>
          <Input v-model="newListName" placeholder="Client Acme" />
        </div>
        <div>
          <label class="block text-sm text-muted mb-1">Description</label>
          <Input v-model="newListDescription" placeholder="All repos for Acme Corp" />
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <Button variant="secondary" @click="showCreateList = false">Cancel</Button>
          <Button variant="primary" :disabled="!newListName.trim()" @click="handleCreateList">
            Create
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Modal: Add Repos ────────────────────────────────────────────────── -->
    <Modal v-model:open="showAddRepos" title="Add Repos to List" size="lg">
      <div class="space-y-3">
        <Input v-model="addRepoSearch" placeholder="Filter repos…" />
        <div class="max-h-80 overflow-y-auto space-y-1">
          <label
            v-for="repo in reposNotInList"
            :key="repo.id"
            class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-white/5 cursor-pointer"
          >
            <input
              type="checkbox"
              :checked="selectedToAdd.has(repo.id)"
              class="accent-primary"
              @change="toggleRepoSelect(repo.id)"
            />
            <span class="font-mono text-xs">{{ repo.fullName }}</span>
          </label>
          <p v-if="reposNotInList.length === 0" class="text-muted text-sm text-center py-4">
            All repos are already in this list, or no repos discovered yet.
          </p>
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <Button variant="secondary" @click="showAddRepos = false; selectedToAdd = new Set()">Cancel</Button>
          <Button
            variant="primary"
            :disabled="selectedToAdd.size === 0"
            @click="handleAddRepos"
          >
            Add {{ selectedToAdd.size > 0 ? selectedToAdd.size : '' }} Repos
          </Button>
        </div>
      </div>
    </Modal>

    <!-- Modal: Export YAML ─────────────────────────────────────────────── -->
    <Modal v-model:open="showExport" title="Exported YAML" size="lg">
      <div class="space-y-3">
        <pre class="bg-surface border border-border rounded-md p-4 text-xs font-mono overflow-auto max-h-80 whitespace-pre-wrap">{{ exportedYaml }}</pre>
        <p class="text-xs text-muted">Copy this YAML and save it to <code>.flotilla/repo-lists/</code> in your project to share with your team.</p>
        <div class="flex justify-end">
          <Button variant="secondary" @click="showExport = false">Close</Button>
        </div>
      </div>
    </Modal>
  </div>
</template>
