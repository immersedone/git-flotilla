import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SecretFinding, LicenceFinding, BranchProtectionStatus } from '@/types/compliance'
import {
  scanSecrets,
  scanLicences,
  auditBranchProtection,
} from '@/services/compliance'

export const useComplianceStore = defineStore('compliance', () => {
  const secretFindings = ref<SecretFinding[]>([])
  const licenceFindings = ref<LicenceFinding[]>([])
  const branchProtection = ref<BranchProtectionStatus[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function scanSecretsAction(repoIds: string[]) {
    isLoading.value = true
    error.value = null
    try {
      secretFindings.value = await scanSecrets(repoIds)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function scanLicencesAction(repoIds: string[], blockedLicences: string[] = []) {
    isLoading.value = true
    error.value = null
    try {
      licenceFindings.value = await scanLicences(repoIds, blockedLicences)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function auditBranchProtectionAction(repoIds: string[]) {
    isLoading.value = true
    error.value = null
    try {
      branchProtection.value = await auditBranchProtection(repoIds)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    secretFindings,
    licenceFindings,
    branchProtection,
    isLoading,
    error,
    scanSecretsAction,
    scanLicencesAction,
    auditBranchProtectionAction,
  }
})
