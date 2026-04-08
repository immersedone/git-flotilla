<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useCveStore } from '@/stores/cve'
import {
  LayoutDashboard,
  FolderGit2,
  ScanSearch,
  Package,
  ShieldAlert,
  Play,
  GitPullRequest,
  Terminal,
  TrendingDown,
  ShieldCheck,
  Settings,
  UserCircle,
} from 'lucide-vue-next'

const route = useRoute()
const cveStore = useCveStore()

const navItems = [
  { name: 'Dashboard',      to: '/dashboard',    icon: LayoutDashboard },
  { name: 'Repositories',   to: '/repos',         icon: FolderGit2 },
  { name: 'Scanner',        to: '/scan',          icon: ScanSearch },
  { name: 'Packages',       to: '/packages',      icon: Package },
  { name: 'CVE Alerts',     to: '/cve',           icon: ShieldAlert, badge: true },
  { name: 'Operations',     to: '/ops',           icon: Play },
  { name: 'PR Queue',       to: '/merge-queue',   icon: GitPullRequest },
  { name: 'Script Runner',  to: '/scripts',       icon: Terminal },
  { name: 'Drift',          to: '/drift',         icon: TrendingDown },
  { name: 'Compliance',     to: '/compliance',    icon: ShieldCheck },
]

const bottomItems = [
  { name: 'Settings', to: '/settings', icon: Settings },
  { name: 'Accounts', to: '/auth',     icon: UserCircle },
]

function isActive(to: string) {
  return route.path.startsWith(to)
}

const badgeCount = computed(() => cveStore.badgeCount)
</script>

<template>
  <nav class="w-52 flex-shrink-0 bg-surface-alt border-r border-border flex flex-col py-4">
    <!-- Logo -->
    <div class="px-4 mb-6">
      <span class="text-primary font-bold text-lg tracking-tight">Git Flotilla</span>
    </div>

    <!-- Main nav -->
    <div class="flex-1 flex flex-col gap-0.5 px-2">
      <RouterLink
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors"
        :class="isActive(item.to)
          ? 'bg-primary/20 text-primary'
          : 'text-muted hover:text-gray-200 hover:bg-white/5'"
      >
        <component :is="item.icon" class="w-4 h-4 flex-shrink-0" />
        <span class="flex-1">{{ item.name }}</span>
        <!-- CVE badge -->
        <span
          v-if="item.badge && badgeCount > 0"
          class="bg-danger text-white text-xs font-bold rounded-full px-1.5 py-0.5 min-w-[1.25rem] text-center"
        >
          {{ badgeCount > 99 ? '99+' : badgeCount }}
        </span>
      </RouterLink>
    </div>

    <!-- Bottom nav -->
    <div class="flex flex-col gap-0.5 px-2 pt-4 border-t border-border mt-4">
      <RouterLink
        v-for="item in bottomItems"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors"
        :class="isActive(item.to)
          ? 'bg-primary/20 text-primary'
          : 'text-muted hover:text-gray-200 hover:bg-white/5'"
      >
        <component :is="item.icon" class="w-4 h-4 flex-shrink-0" />
        <span>{{ item.name }}</span>
      </RouterLink>
    </div>
  </nav>
</template>
