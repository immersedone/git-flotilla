<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useOperationsStore } from '@/stores/operations'
import { useReposStore } from '@/stores/repos'
import type {
  BatchOperation,
  OperationType,
  OperationMode,
  OperationStatus,
} from '@/types/operation'

const opsStore = useOperationsStore()
const reposStore = useReposStore()

// UI state
const showForm = ref(false)
const expandedOpId = ref<string | null>(null)

// Form state
const formType = ref<OperationType>('package_pin')
const formMode = ref<OperationMode>('pin')
const formTargetRepoIds = ref<string[]>([])
const formPackageName = ref('')
const formTargetVersion = ref('')
const formFilePath = ref('')
const formFileContent = ref('')
const formPrTitle = ref('')
const formPrBody = ref('')
const formDryRun = ref(true)
const formSkipCi = ref(false)

const isPackageType = computed(
  () => formType.value === 'package_pin' || formType.value === 'package_bump',
)

const isFileUpdate = computed(() => formType.value === 'file_update')

function resetForm() {
  formType.value = 'package_pin'
  formMode.value = 'pin'
  formTargetRepoIds.value = []
  formPackageName.value = ''
  formTargetVersion.value = ''
  formFilePath.value = ''
  formFileContent.value = ''
  formPrTitle.value = ''
  formPrBody.value = ''
  formDryRun.value = true
  formSkipCi.value = false
}

function openForm() {
  resetForm()
  showForm.value = true
}

function cancelForm() {
  showForm.value = false
}

async function submitForm() {
  try {
    await opsStore.createOp({
      operationType: formType.value,
      mode: isPackageType.value ? formMode.value : undefined,
      targetRepoIds: formTargetRepoIds.value,
      packageName: isPackageType.value ? formPackageName.value : undefined,
      targetVersion: isPackageType.value ? formTargetVersion.value : undefined,
      filePath: isFileUpdate.value ? formFilePath.value : undefined,
      fileContent: isFileUpdate.value ? formFileContent.value : undefined,
      prTitleTemplate: formPrTitle.value || undefined,
      prBodyTemplate: formPrBody.value || undefined,
      isDryRun: formDryRun.value,
      skipCi: formSkipCi.value,
      alsoTargetBranches: [],
      divergenceCheck: false,
    })
    showForm.value = false
  } catch {
    // error is set on the store
  }
}

function toggleExpand(id: string) {
  expandedOpId.value = expandedOpId.value === id ? null : id
}

function statusColour(status: OperationStatus): string {
  const colours: Record<OperationStatus, string> = {
    pending: 'bg-gray-600 text-gray-200',
    running: 'bg-blue-600 text-blue-100',
    completed: 'bg-green-600 text-green-100',
    failed: 'bg-red-600 text-red-100',
    rolled_back: 'bg-amber-600 text-amber-100',
    paused: 'bg-gray-500 text-gray-200',
  }
  return colours[status]
}

function formatDate(iso: string | null): string {
  if (!iso) return '\u2014'
  return new Date(iso).toLocaleString()
}

function typeLabel(type: OperationType): string {
  const labels: Record<OperationType, string> = {
    file_update: 'File Update',
    package_pin: 'Package Pin',
    package_bump: 'Package Bump',
    workflow_sync: 'Workflow Sync',
    script_run: 'Script Run',
    pr_create: 'PR Create',
    commit: 'Commit',
  }
  return labels[type]
}

function progressPercent(op: BatchOperation): number {
  if (
    op.id === expandedOpId.value &&
    opsStore.isRunning &&
    opsStore.progress.total > 0
  ) {
    return Math.round(
      (opsStore.progress.current / opsStore.progress.total) * 100,
    )
  }
  if (op.targetRepoIds.length === 0) return 0
  return Math.round(
    (op.completedRepoIds.length / op.targetRepoIds.length) * 100,
  )
}

function toggleRepoSelection(repoId: string) {
  const idx = formTargetRepoIds.value.indexOf(repoId)
  if (idx === -1) {
    formTargetRepoIds.value.push(repoId)
  } else {
    formTargetRepoIds.value.splice(idx, 1)
  }
}

function selectAllRepos() {
  formTargetRepoIds.value = reposStore.repos.map((r) => r.id)
}

function deselectAllRepos() {
  formTargetRepoIds.value = []
}

onMounted(() => {
  opsStore.loadOperations()
  reposStore.loadRepos()
})
</script>

<template>
  <div class="min-h-full">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-semibold text-white">Operations</h1>
      <button
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded transition-colors"
        @click="openForm"
      >
        New Operation
      </button>
    </div>

    <!-- Error banner -->
    <div
      v-if="opsStore.error"
      class="mb-4 p-3 bg-red-900/30 border border-red-700 rounded text-red-300 text-sm"
    >
      {{ opsStore.error }}
    </div>

    <!-- Creation form -->
    <div
      v-if="showForm"
      class="mb-6 p-6 bg-[#1A1D27] border border-[#2A2D3A] rounded-lg"
    >
      <h2 class="text-lg font-semibold text-white mb-4">
        Create New Operation
      </h2>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Operation type -->
        <div>
          <label class="block text-sm text-gray-400 mb-1">Operation Type</label>
          <select
            v-model="formType"
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white text-sm focus:outline-none focus:border-blue-500"
          >
            <option value="file_update">File Update</option>
            <option value="package_pin">Package Pin</option>
            <option value="package_bump">Package Bump</option>
          </select>
        </div>

        <!-- Mode (package types only) -->
        <div v-if="isPackageType">
          <label class="block text-sm text-gray-400 mb-1">Mode</label>
          <div class="flex gap-4 mt-2">
            <label class="flex items-center gap-2 text-sm text-gray-300">
              <input
                v-model="formMode"
                type="radio"
                value="pin"
                class="accent-blue-500"
              />
              Pin (exact version + overrides)
            </label>
            <label class="flex items-center gap-2 text-sm text-gray-300">
              <input
                v-model="formMode"
                type="radio"
                value="bump"
                class="accent-blue-500"
              />
              Bump (range + remove overrides)
            </label>
          </div>
        </div>

        <!-- Package name (package types only) -->
        <div v-if="isPackageType">
          <label class="block text-sm text-gray-400 mb-1">Package Name</label>
          <input
            v-model="formPackageName"
            type="text"
            placeholder="e.g. lodash"
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white font-mono text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <!-- Target version (package types only) -->
        <div v-if="isPackageType">
          <label class="block text-sm text-gray-400 mb-1"
            >Target Version</label
          >
          <input
            v-model="formTargetVersion"
            type="text"
            placeholder="e.g. 4.17.21"
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white font-mono text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <!-- File path (file_update only) -->
        <div v-if="isFileUpdate">
          <label class="block text-sm text-gray-400 mb-1">File Path</label>
          <input
            v-model="formFilePath"
            type="text"
            placeholder="e.g. .nvmrc"
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white font-mono text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <!-- File content (file_update only) -->
        <div v-if="isFileUpdate" class="md:col-span-2">
          <label class="block text-sm text-gray-400 mb-1">File Content</label>
          <textarea
            v-model="formFileContent"
            rows="4"
            placeholder="File content to push..."
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white font-mono text-sm focus:outline-none focus:border-blue-500 resize-y"
          />
        </div>

        <!-- PR title -->
        <div>
          <label class="block text-sm text-gray-400 mb-1"
            >PR Title Template</label
          >
          <input
            v-model="formPrTitle"
            type="text"
            placeholder="e.g. chore: pin {{PACKAGE}} to {{VERSION}}"
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <!-- PR body -->
        <div>
          <label class="block text-sm text-gray-400 mb-1"
            >PR Body Template</label
          >
          <input
            v-model="formPrBody"
            type="text"
            placeholder="Markdown body..."
            class="w-full px-3 py-2 bg-[#0F1117] border border-[#2A2D3A] rounded text-white text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <!-- Target repos -->
        <div class="md:col-span-2">
          <div class="flex items-center justify-between mb-1">
            <label class="text-sm text-gray-400">Target Repositories</label>
            <div class="flex gap-2">
              <button
                class="text-xs text-blue-400 hover:text-blue-300"
                @click="selectAllRepos"
              >
                Select All
              </button>
              <button
                class="text-xs text-gray-400 hover:text-gray-300"
                @click="deselectAllRepos"
              >
                Deselect All
              </button>
            </div>
          </div>
          <div
            class="max-h-48 overflow-y-auto border border-[#2A2D3A] rounded bg-[#0F1117] p-2"
          >
            <div
              v-if="reposStore.repos.length === 0"
              class="text-sm text-gray-500 p-2"
            >
              No repos loaded. Go to Repos to discover repositories.
            </div>
            <label
              v-for="repo in reposStore.repos"
              :key="repo.id"
              class="flex items-center gap-2 px-2 py-1 hover:bg-[#1A1D27] rounded cursor-pointer"
            >
              <input
                type="checkbox"
                :checked="formTargetRepoIds.includes(repo.id)"
                class="accent-blue-500"
                @change="toggleRepoSelection(repo.id)"
              />
              <span class="text-sm text-gray-300 font-mono">{{
                repo.fullName
              }}</span>
            </label>
          </div>
          <p class="mt-1 text-xs text-gray-500">
            {{ formTargetRepoIds.length }} selected
          </p>
        </div>

        <!-- Options row -->
        <div class="md:col-span-2 flex gap-6">
          <label class="flex items-center gap-2 text-sm text-gray-300">
            <input
              v-model="formDryRun"
              type="checkbox"
              class="accent-blue-500"
            />
            Dry Run
          </label>
          <label class="flex items-center gap-2 text-sm text-gray-300">
            <input
              v-model="formSkipCi"
              type="checkbox"
              class="accent-blue-500"
            />
            Skip CI
          </label>
        </div>
      </div>

      <!-- Form actions -->
      <div class="flex gap-3 mt-6">
        <button
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          :disabled="formTargetRepoIds.length === 0"
          @click="submitForm"
        >
          Create Operation
        </button>
        <button
          class="px-4 py-2 bg-[#2A2D3A] hover:bg-[#353845] text-gray-300 text-sm font-medium rounded transition-colors"
          @click="cancelForm"
        >
          Cancel
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div
      v-if="opsStore.isLoading"
      class="text-gray-400 text-sm py-8 text-center"
    >
      Loading operations...
    </div>

    <!-- Empty state -->
    <div
      v-else-if="opsStore.operations.length === 0 && !showForm"
      class="text-center py-16"
    >
      <p class="text-gray-500 text-lg mb-2">No operations yet.</p>
      <p class="text-gray-600 text-sm">Create one to get started.</p>
    </div>

    <!-- Operations list -->
    <div v-else class="space-y-2">
      <div
        v-for="op in opsStore.operations"
        :key="op.id"
        class="bg-[#1A1D27] border border-[#2A2D3A] rounded-lg overflow-hidden"
      >
        <!-- Row -->
        <div
          class="flex items-center gap-4 px-4 py-3 cursor-pointer hover:bg-[#1E2130] transition-colors"
          @click="toggleExpand(op.id)"
        >
          <span
            class="text-xs font-medium px-2 py-0.5 rounded"
            :class="statusColour(op.status)"
          >
            {{ op.status }}
          </span>
          <span class="text-sm text-gray-300">{{ typeLabel(op.type) }}</span>
          <span
            v-if="op.mode"
            class="text-xs text-gray-500 font-mono"
          >
            {{ op.mode }}
          </span>
          <span class="text-xs text-gray-500 ml-auto">
            {{ op.targetRepoIds.length }} repos
          </span>
          <span class="text-xs text-gray-600 font-mono">
            {{ formatDate(op.createdAt) }}
          </span>
          <span v-if="op.completedAt" class="text-xs text-gray-600 font-mono">
            {{ formatDate(op.completedAt) }}
          </span>
          <span
            v-if="op.isDryRun"
            class="text-xs bg-amber-900/40 text-amber-400 px-1.5 py-0.5 rounded"
          >
            dry run
          </span>
          <svg
            class="w-4 h-4 text-gray-500 transition-transform"
            :class="{ 'rotate-180': expandedOpId === op.id }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </div>

        <!-- Expanded detail -->
        <div
          v-if="expandedOpId === op.id"
          class="border-t border-[#2A2D3A] px-4 py-4"
        >
          <!-- Progress bar (if running) -->
          <div v-if="op.status === 'running' && opsStore.isRunning" class="mb-4">
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs text-gray-400">Progress</span>
              <span class="text-xs text-gray-400 font-mono"
                >{{ opsStore.progress.current }}/{{
                  opsStore.progress.total
                }}
                ({{ progressPercent(op) }}%)</span
              >
            </div>
            <div class="w-full h-2 bg-[#0F1117] rounded-full overflow-hidden">
              <div
                class="h-full bg-blue-500 rounded-full transition-all duration-300"
                :style="{ width: progressPercent(op) + '%' }"
              />
            </div>
          </div>

          <!-- Action buttons -->
          <div class="flex gap-2 mb-4">
            <button
              v-if="op.status === 'pending'"
              class="px-3 py-1.5 bg-green-700 hover:bg-green-600 text-white text-xs font-medium rounded transition-colors"
              @click.stop="opsStore.runOp(op.id)"
            >
              Run
            </button>
            <button
              v-if="op.status === 'running'"
              class="px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white text-xs font-medium rounded transition-colors"
              @click.stop="opsStore.abortOp(op.id)"
            >
              Abort
            </button>
            <button
              v-if="op.status === 'completed'"
              class="px-3 py-1.5 bg-amber-700 hover:bg-amber-600 text-white text-xs font-medium rounded transition-colors"
              @click.stop="opsStore.rollbackOp(op.id)"
            >
              Rollback
            </button>
          </div>

          <!-- Per-repo results -->
          <div v-if="op.results.length > 0">
            <h3 class="text-sm font-medium text-gray-300 mb-2">Results</h3>
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="text-left text-xs text-gray-500 border-b border-[#2A2D3A]">
                    <th class="pb-2 pr-4">Repo</th>
                    <th class="pb-2 pr-4">Status</th>
                    <th class="pb-2 pr-4">PR</th>
                    <th class="pb-2 pr-4">Error</th>
                    <th class="pb-2">Diff</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="result in op.results"
                    :key="result.repoId"
                    class="border-b border-[#2A2D3A]/50"
                  >
                    <td class="py-2 pr-4 font-mono text-gray-300">
                      {{ result.repoId }}
                    </td>
                    <td class="py-2 pr-4 text-gray-400">
                      {{ result.status }}
                    </td>
                    <td class="py-2 pr-4">
                      <a
                        v-if="result.prUrl"
                        :href="result.prUrl"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-blue-400 hover:text-blue-300 underline"
                        @click.stop
                      >
                        View PR
                      </a>
                      <span v-else class="text-gray-600">&mdash;</span>
                    </td>
                    <td class="py-2 pr-4">
                      <span
                        v-if="result.error"
                        class="text-red-400 text-xs"
                      >
                        {{ result.error }}
                      </span>
                      <span v-else class="text-gray-600">&mdash;</span>
                    </td>
                    <td class="py-2">
                      <pre
                        v-if="result.diff"
                        class="text-xs text-gray-400 font-mono bg-[#0F1117] p-2 rounded max-h-32 overflow-auto whitespace-pre-wrap"
                        >{{ result.diff }}</pre
                      >
                      <span v-else class="text-gray-600">&mdash;</span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Live repo statuses (while running) -->
          <div
            v-else-if="
              op.status === 'running' &&
              Object.keys(opsStore.repoStatuses).length > 0
            "
          >
            <h3 class="text-sm font-medium text-gray-300 mb-2">
              Live Progress
            </h3>
            <div class="space-y-1">
              <div
                v-for="(repoStatus, repoId) in opsStore.repoStatuses"
                :key="repoId"
                class="flex items-center gap-3 text-xs"
              >
                <span class="font-mono text-gray-300 w-48 truncate">{{
                  repoId
                }}</span>
                <span class="text-gray-400">{{ repoStatus.status }}</span>
                <span
                  v-if="repoStatus.error"
                  class="text-red-400"
                >
                  {{ repoStatus.error }}
                </span>
              </div>
            </div>
          </div>

          <p
            v-else
            class="text-gray-600 text-xs"
          >
            No results yet.
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
