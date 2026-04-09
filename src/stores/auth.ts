import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AccountInfo } from '@/services/auth'
import { listAccounts, addAccount, removeAccount } from '@/services/auth'

export const useAuthStore = defineStore('auth', () => {
  const accounts  = ref<AccountInfo[]>([])
  const isLoading = ref(false)
  const error     = ref<string | null>(null)

  const hasAccounts   = computed(() => accounts.value.length > 0)
  const githubAccount = computed(() => accounts.value.find(a => a.provider === 'github') ?? null)
  const gitlabAccount = computed(() => accounts.value.find(a => a.provider === 'gitlab') ?? null)

  async function loadAccounts() {
    isLoading.value = true
    error.value = null
    try {
      accounts.value = await listAccounts()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function addAccountAction(provider: 'github' | 'gitlab', token: string) {
    error.value = null
    try {
      const account = await addAccount(provider, token)
      accounts.value.push(account)
      return account
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function removeAccountAction(id: string) {
    error.value = null
    try {
      await removeAccount(id)
      accounts.value = accounts.value.filter(a => a.id !== id)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    accounts, isLoading, error,
    hasAccounts, githubAccount, gitlabAccount,
    loadAccounts, addAccountAction, removeAccountAction,
  }
})
