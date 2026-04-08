import { invoke } from '@tauri-apps/api/core'

export interface AccountInfo {
  id: string
  provider: string
  username: string
  avatarUrl: string | null
  scopes: string[]
  missingScopes: string[]
}

export function addAccount(provider: string, token: string): Promise<AccountInfo> {
  return invoke('add_account', { provider, token })
}

export function removeAccount(id: string): Promise<void> {
  return invoke('remove_account', { id })
}

export function listAccounts(): Promise<AccountInfo[]> {
  return invoke('list_accounts')
}

export function validateToken(provider: string, token: string): Promise<AccountInfo> {
  return invoke('validate_token', { provider, token })
}
