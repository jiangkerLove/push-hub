import { createRouter, createWebHistory } from 'vue-router'
import { getToken } from '@/api/client'
import { LAST_APP_KEY, LEGACY_ALL_APP_ID, sanitizeStoredAppId } from '@/stores/app'

const appDetail = () => import('@/views/apps/AppDetailView.vue')

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { public: true },
    },
    {
      path: '/',
      component: () => import('@/layouts/AdminLayout.vue'),
      children: [
        { path: '', redirect: () => {
          const lastId = sanitizeStoredAppId(localStorage.getItem(LAST_APP_KEY))
          if (lastId) return `/apps/${lastId}/send`
          return '/apps'
        } },
        {
          path: 'apps',
          name: 'apps',
          component: () => import('@/views/apps/AppListView.vue'),
        },
        {
          path: 'accounts',
          name: 'accounts',
          component: () => import('@/views/accounts/AccountManageView.vue'),
        },
        {
          path: 'apps/:id',
          redirect: (to) => `/apps/${to.params.id}/send`,
        },
        {
          path: 'apps/:id/send',
          name: 'app-send',
          component: appDetail,
          meta: { section: 'send' },
        },
        {
          path: 'apps/:id/templates',
          name: 'app-templates',
          component: appDetail,
          meta: { section: 'templates' },
        },
        {
          path: 'apps/:id/channels',
          name: 'app-channels',
          component: appDetail,
          meta: { section: 'channels' },
        },
        {
          path: 'apps/:id/devices',
          name: 'app-devices',
          component: appDetail,
          meta: { section: 'devices' },
        },
        {
          path: 'apps/:id/jobs',
          name: 'app-jobs',
          component: appDetail,
          meta: { section: 'jobs' },
        },
        {
          path: 'apps/:id/stats',
          name: 'app-stats',
          component: appDetail,
          meta: { section: 'stats' },
        },
        {
          path: 'apps/:id/config',
          name: 'app-config',
          component: appDetail,
          meta: { section: 'config' },
        },
        {
          path: 'apps/:id/integrate',
          name: 'app-integrate',
          component: appDetail,
          meta: { section: 'integrate' },
        },
      ],
    },
  ],
})

router.beforeEach((to) => {
  if (to.meta.public) return true
  if (!getToken()) return { name: 'login', query: { redirect: to.fullPath } }

  if (to.path === '/') {
    const lastId = sanitizeStoredAppId(localStorage.getItem(LAST_APP_KEY))
    if (lastId) return { path: `/apps/${lastId}/send`, replace: true }
    return { path: '/apps', replace: true }
  }

  if (to.params.id === LEGACY_ALL_APP_ID) {
    localStorage.removeItem(LAST_APP_KEY)
    return { path: '/apps', replace: true }
  }

  return true
})

export default router
