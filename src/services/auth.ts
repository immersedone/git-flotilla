import { invoke } from './tauri'

export interface AccountInfo {
  id: string
  provider: 'github' | 'gitlab'
  username: string
  avatarUrl: string | null
  scopes: string[]
  missingScopes: string[]
}

export function addAccount(provider: 'github' | 'gitlab', token: string): Promise<AccountInfo> {
  return invoke('add_account', { provider, token })
}

export function removeAccount(id: string): Promise<void> {
  return invoke('remove_account', { id })
}

export function listAccounts(): Promise<AccountInfo[]> {
  return invoke('list_accounts')
}

export function validateToken(provider: 'github' | 'gitlab', token: string): Promise<AccountInfo> {
  return invoke('validate_token', { provider, token })
}
