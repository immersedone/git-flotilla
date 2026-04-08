import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SecretFinding, LicenceFinding, BranchProtectionStatus } from '@/types/compliance'

export const useComplianceStore = defineStore('compliance', () => {
  const secretFindings = ref<SecretFinding[]>([])
  const licenceFindings = ref<LicenceFinding[]>([])
  const branchProtection = ref<BranchProtectionStatus[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  return { secretFindings, licenceFindings, branchProtection, isLoading, error }
})
