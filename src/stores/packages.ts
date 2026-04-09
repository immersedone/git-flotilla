import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DependencyMatrix, ChangelogEntry, Ecosystem } from '@/types/package'
import { getDependencyMatrix, getPackageChangelog, exportMatrixCsv } from '@/services/packages'

export const usePackagesStore = defineStore('packages', () => {
  const matrix = ref<DependencyMatrix | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedEcosystem = ref<string | null>(null)
  const selectedRepoListId = ref<string | null>(null)
  const changelog = ref<ChangelogEntry[]>([])
  const changelogLoading = ref(false)
  const searchQuery = ref('')

  const filteredPackages = computed(() => {
    if (!matrix.value) return []
    let packages = matrix.value.packages
    if (selectedEcosystem.value) {
      packages = packages.filter(p => p.ecosystem === selectedEcosystem.value)
    }
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      packages = packages.filter(p => p.name.toLowerCase().includes(query))
    }
    return packages
  })

  const ecosystems = computed<Ecosystem[]>(() => {
    if (!matrix.value) return []
    const set = new Set<Ecosystem>()
    for (const pkg of matrix.value.packages) {
      set.add(pkg.ecosystem)
    }
    return [...set].sort()
  })

  const driftCount = computed(() => {
    if (!matrix.value) return 0
    return matrix.value.packages.filter(p => p.hasDrift).length
  })

  async function loadMatrix(repoListId?: string, ecosystem?: string) {
    isLoading.value = true
    error.value = null
    try {
      matrix.value = await getDependencyMatrix(repoListId, ecosystem)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function fetchChangelog(
    packageName: string,
    ecosystem: string,
    fromVersion: string,
    toVersion: string,
  ) {
    changelogLoading.value = true
    error.value = null
    try {
      changelog.value = await getPackageChangelog(packageName, ecosystem, fromVersion, toVersion)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      changelogLoading.value = false
    }
  }

  async function exportCsv(repoListId?: string): Promise<string> {
    error.value = null
    try {
      return await exportMatrixCsv(repoListId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    matrix,
    isLoading,
    error,
    selectedEcosystem,
    selectedRepoListId,
    changelog,
    changelogLoading,
    searchQuery,
    filteredPackages,
    ecosystems,
    driftCount,
    loadMatrix,
    fetchChangelog,
    exportCsv,
  }
})
