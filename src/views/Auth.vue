<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { validateToken } from '@/services/auth'
import type { AccountInfo } from '@/services/auth'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'

const authStore = useAuthStore()

onMounted(() => { authStore.loadAccounts().catch(() => {}) })

// ── Add account form ─────────────────────────────────────────────────────
const provider  = ref<'github' | 'gitlab'>('github')
const token     = ref('')
const preview   = ref<AccountInfo | null>(null)
const validating = ref(false)
const adding    = ref(false)
const formError = ref<string | null>(null)

async function handleValidate() {
  if (!token.value.trim()) return
  validating.value = true
  formError.value  = null
  preview.value    = null
  try {
    preview.value = await validateToken(provider.value, token.value.trim())
  } catch (e) {
    formError.value = String(e)
  } finally {
    validating.value = false
  }
}

async function handleAdd() {
  if (!preview.value) return
  adding.value    = true
  formError.value = null
  try {
    await authStore.addAccountAction(provider.value, token.value.trim())
    token.value   = ''
    preview.value = null
  } catch (e) {
    formError.value = String(e)
  } finally {
    adding.value = false
  }
}

const removingId = ref<string | null>(null)

async function handleRemove(id: string) {
  if (removingId.value) return
  removingId.value = id
  try {
    await authStore.removeAccountAction(id)
  } catch (e) {
    formError.value = String(e)
  } finally {
    removingId.value = null
  }
}

const REQUIRED_SCOPES: Record<string, string[]> = {
  github: ['repo', 'workflow', 'read:org'],
  gitlab: ['api', 'read_repository', 'write_repository'],
}

const TOKEN_PLACEHOLDER: Record<string, string> = {
  github: 'ghp_xxxxxxxxxxxxxxxx',
  gitlab: 'glpat-xxxxxxxxxxxxxxxxxxxx',
}
</script>

<template>
  <div class="p-6 max-w-2xl space-y-8">
    <div>
      <h1 class="text-2xl font-semibold mb-1">Accounts</h1>
      <p class="text-muted text-sm">Connect GitHub and GitLab accounts via Personal Access Token.</p>
    </div>

    <!-- Connected accounts list -->
    <section v-if="authStore.accounts.length > 0">
      <h2 class="text-sm font-semibold text-muted uppercase tracking-wider mb-3">Connected</h2>
      <div class="space-y-3">
        <div
          v-for="account in authStore.accounts"
          :key="account.id"
          class="bg-surface-alt border border-border rounded-lg p-4 flex items-center gap-4"
        >
          <div class="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center shrink-0">
            <span class="text-primary text-sm font-bold">{{ account.username[0].toUpperCase() }}</span>
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm">{{ account.username }}</span>
              <span class="text-xs text-muted px-1.5 py-0.5 bg-surface rounded border border-border capitalize">
                {{ account.provider }}
              </span>
            </div>
            <!-- Missing scopes warning -->
            <div v-if="account.missingScopes.length > 0" class="mt-1 text-xs text-warning">
              Missing scopes: {{ account.missingScopes.join(', ') }}
            </div>
          </div>
          <Button variant="danger" size="sm" :disabled="removingId === account.id" @click="handleRemove(account.id)">Remove</Button>
        </div>
      </div>
    </section>

    <div v-else-if="authStore.isLoading" class="text-muted text-sm">Loading accounts…</div>

    <div v-else class="text-muted text-sm">No accounts connected yet.</div>

    <!-- Add account form -->
    <section class="bg-surface-alt border border-border rounded-lg p-6 space-y-4">
      <h2 class="text-base font-semibold">Add Account</h2>

      <div class="flex gap-2">
        <button
          class="flex-1 py-1.5 text-sm rounded-md border transition-colors"
          :class="provider === 'github'
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border text-muted hover:border-primary/50'"
          @click="provider = 'github'"
        >GitHub</button>
        <button
          class="flex-1 py-1.5 text-sm rounded-md border transition-colors"
          :class="provider === 'gitlab'
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border text-muted hover:border-primary/50'"
          @click="provider = 'gitlab'"
        >GitLab</button>
      </div>

      <div>
        <label class="block text-sm text-muted mb-1">Personal Access Token</label>
        <Input
          v-model="token"
          type="password"
          :placeholder="TOKEN_PLACEHOLDER[provider] ?? 'Enter token'"
          :error="formError ?? undefined"
        />
        <p class="text-xs text-muted mt-1">
          Required scopes: {{ REQUIRED_SCOPES[provider]?.join(', ') ?? 'none' }}
        </p>
      </div>

      <!-- Validation preview -->
      <div v-if="preview" class="bg-surface rounded-md border border-border p-3 space-y-1">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-success" />
          <span class="text-sm font-medium">{{ preview.username }}</span>
        </div>
        <div class="text-xs text-muted">
          Scopes: {{ preview.scopes.join(', ') || 'none' }}
        </div>
        <div v-if="preview.missingScopes.length > 0" class="text-xs text-warning">
          Warning: missing {{ preview.missingScopes.join(', ') }} — some features may not work
        </div>
      </div>

      <div class="flex gap-3">
        <Button
          variant="secondary"
          :loading="validating"
          :disabled="!token.trim() || validating"
          @click="handleValidate"
        >
          Validate
        </Button>
        <Button
          variant="primary"
          :loading="adding"
          :disabled="!preview || adding"
          @click="handleAdd"
        >
          Add Account
        </Button>
      </div>
    </section>
  </div>
</template>
