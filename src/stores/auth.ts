import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AccountInfo } from '@/services/auth'

export const useAuthStore = defineStore('auth', () => {
  const accounts = ref<AccountInfo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const hasAccounts = computed(() => accounts.value.length > 0)
  const githubAccount = computed(() => accounts.value.find(a => a.provider === 'github'))
  const gitlabAccount = computed(() => accounts.value.find(a => a.provider === 'gitlab'))

  return { accounts, isLoading, error, hasAccounts, githubAccount, gitlabAccount }
})
