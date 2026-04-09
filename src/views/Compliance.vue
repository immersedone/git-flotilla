<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useComplianceStore } from '@/stores/compliance'
import { useReposStore } from '@/stores/repos'

const complianceStore = useComplianceStore()
const reposStore = useReposStore()

type TabId = 'secrets' | 'licences' | 'branch-protection'
const activeTab = ref<TabId>('secrets')

const tabs: { id: TabId; label: string }[] = [
  { id: 'secrets', label: 'Secrets' },
  { id: 'licences', label: 'Licences' },
  { id: 'branch-protection', label: 'Branch Protection' },
]

onMounted(() => {
  reposStore.loadRepos().catch(() => {})
})

function allRepoIds(): string[] {
  return reposStore.repos.map(r => r.id)
}

async function handleScanSecrets() {
  await complianceStore.scanSecretsAction(allRepoIds())
}

async function handleScanLicences() {
  await complianceStore.scanLicencesAction(allRepoIds())
}

async function handleAuditBranchProtection() {
  await complianceStore.auditBranchProtectionAction(allRepoIds())
}

function severityClass(secretType: string): string {
  const lower = secretType.toLowerCase()
  if (lower.includes('key') || lower.includes('token') || lower.includes('password')) {
    return 'text-danger'
  }
  return 'text-warning'
}

function complianceStatusClass(isCompliant: boolean): string {
  return isCompliant ? 'bg-success/20 text-success' : 'bg-danger/20 text-danger'
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-semibold">Compliance</h1>

    <p v-if="complianceStore.error" class="text-danger text-sm">{{ complianceStore.error }}</p>

    <!-- Tabs -->
    <div class="flex border-b border-border">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px"
        :class="activeTab === tab.id
          ? 'border-primary text-primary'
          : 'border-transparent text-muted hover:text-white'"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- Secrets tab -->
    <div v-if="activeTab === 'secrets'" class="space-y-4">
      <div class="flex items-center justify-between">
        <p class="text-sm text-muted">Scan repositories for exposed secrets and credentials.</p>
        <button
          class="px-4 py-2 rounded-md bg-primary text-white text-sm font-medium hover:bg-primary/80 transition-colors"
          :disabled="complianceStore.isLoading || reposStore.repos.length === 0"
          @click="handleScanSecrets"
        >
          {{ complianceStore.isLoading ? 'Scanning...' : 'Scan Secrets' }}
        </button>
      </div>

      <div
        v-if="complianceStore.secretFindings.length === 0 && !complianceStore.isLoading"
        class="rounded-lg bg-surface-alt border border-border p-8 text-center text-muted"
      >
        <p class="text-lg font-medium mb-1">No secret findings</p>
        <p class="text-sm">Run a scan to check for exposed secrets.</p>
      </div>

      <div v-else-if="complianceStore.secretFindings.length > 0" class="rounded-lg bg-surface-alt border border-border overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted">
              <th class="px-4 py-2 font-medium">Repository</th>
              <th class="px-4 py-2 font-medium">File</th>
              <th class="px-4 py-2 font-medium">Line</th>
              <th class="px-4 py-2 font-medium">Type</th>
              <th class="px-4 py-2 font-medium">Pattern</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(finding, idx) in complianceStore.secretFindings"
              :key="`${finding.repoId}:${finding.filePath}:${finding.lineNumber}:${idx}`"
              class="border-b border-border last:border-b-0 hover:bg-surface/50"
            >
              <td class="px-4 py-2 font-mono text-xs">{{ finding.repoId }}</td>
              <td class="px-4 py-2 font-mono text-xs">{{ finding.filePath }}</td>
              <td class="px-4 py-2 text-xs text-muted">{{ finding.lineNumber }}</td>
              <td class="px-4 py-2">
                <span class="text-xs font-mono" :class="severityClass(finding.secretType)">
                  {{ finding.secretType }}
                </span>
              </td>
              <td class="px-4 py-2 font-mono text-xs text-muted truncate max-w-xs">
                {{ finding.matchedPattern }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Licences tab -->
    <div v-if="activeTab === 'licences'" class="space-y-4">
      <div class="flex items-center justify-between">
        <p class="text-sm text-muted">Audit dependency licences across repositories.</p>
        <button
          class="px-4 py-2 rounded-md bg-primary text-white text-sm font-medium hover:bg-primary/80 transition-colors"
          :disabled="complianceStore.isLoading || reposStore.repos.length === 0"
          @click="handleScanLicences"
        >
          {{ complianceStore.isLoading ? 'Scanning...' : 'Scan Licences' }}
        </button>
      </div>

      <div
        v-if="complianceStore.licenceFindings.length === 0 && !complianceStore.isLoading"
        class="rounded-lg bg-surface-alt border border-border p-8 text-center text-muted"
      >
        <p class="text-lg font-medium mb-1">No licence findings</p>
        <p class="text-sm">Run a scan to audit dependency licences.</p>
      </div>

      <div v-else-if="complianceStore.licenceFindings.length > 0" class="rounded-lg bg-surface-alt border border-border overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted">
              <th class="px-4 py-2 font-medium">Repository</th>
              <th class="px-4 py-2 font-medium">Package</th>
              <th class="px-4 py-2 font-medium">Ecosystem</th>
              <th class="px-4 py-2 font-medium">Licence</th>
              <th class="px-4 py-2 font-medium">Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(finding, idx) in complianceStore.licenceFindings"
              :key="`${finding.repoId}:${finding.packageName}:${idx}`"
              class="border-b border-border last:border-b-0 hover:bg-surface/50"
            >
              <td class="px-4 py-2 font-mono text-xs">{{ finding.repoId }}</td>
              <td class="px-4 py-2 font-mono text-xs">{{ finding.packageName }}</td>
              <td class="px-4 py-2 text-xs text-muted">{{ finding.ecosystem }}</td>
              <td class="px-4 py-2 font-mono text-xs">{{ finding.licence }}</td>
              <td class="px-4 py-2">
                <span
                  class="inline-block rounded px-2 py-0.5 text-xs font-mono"
                  :class="finding.isFlagged ? 'bg-danger/20 text-danger' : 'bg-success/20 text-success'"
                >
                  {{ finding.isFlagged ? 'Flagged' : 'Permissive' }}
                </span>
                <span v-if="finding.flagReason" class="ml-2 text-xs text-muted">
                  {{ finding.flagReason }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Branch Protection tab -->
    <div v-if="activeTab === 'branch-protection'" class="space-y-4">
      <div class="flex items-center justify-between">
        <p class="text-sm text-muted">Audit branch protection rules across repositories.</p>
        <button
          class="px-4 py-2 rounded-md bg-primary text-white text-sm font-medium hover:bg-primary/80 transition-colors"
          :disabled="complianceStore.isLoading || reposStore.repos.length === 0"
          @click="handleAuditBranchProtection"
        >
          {{ complianceStore.isLoading ? 'Auditing...' : 'Audit Protection' }}
        </button>
      </div>

      <div
        v-if="complianceStore.branchProtection.length === 0 && !complianceStore.isLoading"
        class="rounded-lg bg-surface-alt border border-border p-8 text-center text-muted"
      >
        <p class="text-lg font-medium mb-1">No audit results</p>
        <p class="text-sm">Run an audit to check branch protection status.</p>
      </div>

      <div v-else-if="complianceStore.branchProtection.length > 0" class="rounded-lg bg-surface-alt border border-border overflow-hidden">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border text-left text-muted">
              <th class="px-4 py-2 font-medium">Repository</th>
              <th class="px-4 py-2 font-medium">Branch</th>
              <th class="px-4 py-2 font-medium">Reviews</th>
              <th class="px-4 py-2 font-medium">Status Checks</th>
              <th class="px-4 py-2 font-medium">Push Restrictions</th>
              <th class="px-4 py-2 font-medium">Compliant</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(status, idx) in complianceStore.branchProtection"
              :key="`${status.repoId}:${status.branch}:${idx}`"
              class="border-b border-border last:border-b-0 hover:bg-surface/50"
            >
              <td class="px-4 py-2 font-mono text-xs">{{ status.repoId }}</td>
              <td class="px-4 py-2 font-mono text-xs">{{ status.branch }}</td>
              <td class="px-4 py-2">
                <span :class="status.requiresReviews ? 'text-success' : 'text-danger'" class="text-xs">
                  {{ status.requiresReviews ? 'Yes' : 'No' }}
                </span>
              </td>
              <td class="px-4 py-2">
                <span :class="status.requiresStatusChecks ? 'text-success' : 'text-danger'" class="text-xs">
                  {{ status.requiresStatusChecks ? 'Yes' : 'No' }}
                </span>
              </td>
              <td class="px-4 py-2">
                <span :class="status.restrictsPushes ? 'text-success' : 'text-danger'" class="text-xs">
                  {{ status.restrictsPushes ? 'Yes' : 'No' }}
                </span>
              </td>
              <td class="px-4 py-2">
                <span
                  class="inline-block rounded px-2 py-0.5 text-xs font-mono"
                  :class="complianceStatusClass(status.isCompliant)"
                >
                  {{ status.isCompliant ? 'Compliant' : 'Non-compliant' }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
