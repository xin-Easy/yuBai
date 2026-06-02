import { createRouter, createWebHashHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('../pages/dashboard/index.vue'),
      meta: { title: '控制台', description: '查看迁移进度、核心指标和当前运行状态' },
    },
    {
      path: '/browser/list',
      name: 'browser-list',
      component: () => import('../pages/browser-list/index.vue'),
      meta: { title: '实例列表', description: '集中管理浏览器实例、内核绑定和运行状态' },
    },
    {
      path: '/browser/automation',
      name: 'automation',
      component: () => import('../pages/automation/index.vue'),
      meta: { title: '自动化脚本', description: '查看运行时状态、脚本数量和最近执行情况' },
    },
    {
      path: '/browser/cores',
      name: 'browser-cores',
      component: () => import('../pages/core-management/index.vue'),
      meta: { title: '内核管理', description: '统一管理已安装内核，并从本机或在线来源快速添加' },
    },
    {
      path: '/browser/proxy-pool',
      name: 'proxy-pool',
      component: () => import('../pages/proxy-pool/index.vue'),
      meta: { title: '代理池配置', description: '导入、测试和维护实例使用的代理节点' },
    },
    {
      path: '/browser/bookmarks',
      name: 'bookmarks',
      component: () => import('../pages/bookmark-settings/index.vue'),
      meta: { title: '默认书签', description: '维护实例初始化时使用的默认书签集合' },
    },
    {
      path: '/browser/tags',
      name: 'browser-tags',
      component: () => import('../pages/tag-management/index.vue'),
      meta: { title: '标签管理', description: '管理实例分组并汇总当前已使用的标签' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../pages/settings/index.vue'),
      meta: { title: '系统设置', description: '调整默认参数、目录位置和浏览器启动行为' },
    },
    {
      path: '/system/docs',
      name: 'docs',
      component: () => import('../pages/feature-stub/index.vue'),
      meta: { title: '文档中心', description: '后续会补齐 Launch API、使用说明和排障文档' },
    },
    {
      path: '/browser/logs',
      name: 'logs',
      component: () => import('../pages/logs/index.vue'),
      meta: { title: '日志查看', description: '查看当前应用日志和调试输出' },
    },
  ],
})
