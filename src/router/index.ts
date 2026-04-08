import 'vue-router'

declare module 'vue-router' {
  interface RouteMeta {
    title?: string
  }
}

import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/dashboard',
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('@/views/Dashboard.vue'),
      meta: { title: 'Dashboard' },
    },
    {
      path: '/repos',
      name: 'repos',
      component: () => import('@/views/RepoLists.vue'),
      meta: { title: 'Repositories' },
    },
    {
      path: '/scan',
      name: 'scan',
      component: () => import('@/views/Scanner.vue'),
      meta: { title: 'Scanner' },
    },
    {
      path: '/packages',
      name: 'packages',
      component: () => import('@/views/Packages.vue'),
      meta: { title: 'Packages' },
    },
    {
      path: '/cve',
      name: 'cve',
      component: () => import('@/views/CVEAlerts.vue'),
      meta: { title: 'CVE Alerts' },
    },
    {
      path: '/cve/:id',
      name: 'cve-incident',
      component: () => import('@/views/CVEIncident.vue'),
      meta: { title: 'CVE Incident' },
    },
    {
      path: '/ops',
      name: 'ops',
      component: () => import('@/views/Operations.vue'),
      meta: { title: 'Operations' },
    },
    {
      path: '/merge-queue',
      name: 'merge-queue',
      component: () => import('@/views/MergeQueue.vue'),
      meta: { title: 'PR Merge Queue' },
    },
    {
      path: '/scripts',
      name: 'scripts',
      component: () => import('@/views/ScriptRunner.vue'),
      meta: { title: 'Script Runner' },
    },
    {
      path: '/drift',
      name: 'drift',
      component: () => import('@/views/DriftDashboard.vue'),
      meta: { title: 'Drift Dashboard' },
    },
    {
      path: '/compliance',
      name: 'compliance',
      component: () => import('@/views/Compliance.vue'),
      meta: { title: 'Compliance' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
      meta: { title: 'Settings' },
    },
    {
      path: '/auth',
      name: 'auth',
      component: () => import('@/views/Auth.vue'),
      meta: { title: 'Accounts' },
    },
  ],
})

router.afterEach((to) => {
  document.title = to.meta.title
    ? `${to.meta.title} — Git Flotilla`
    : 'Git Flotilla'
})

export default router
