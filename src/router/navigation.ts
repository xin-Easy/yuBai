import {
  Aim,
  Collection,
  Cpu,
  Document,
  Files,
  House,
  Link,
  Monitor,
  Setting,
  Tickets,
} from '@element-plus/icons-vue'
import type { Component } from 'vue'

export interface NavItem {
  name: string
  path: string
  icon: Component
}

export interface NavSection {
  title: string
  items: NavItem[]
}

export const projectConfig = {
  name: 'yubai',
  shortName: 'YB',
  description: '面向多账号隔离、代理绑定和本地浏览器环境管理的桌面工作台',
}

export const navigationConfig: NavSection[] = [
  {
    title: '总览',
    items: [{ name: '控制台', path: '/', icon: House }],
  },
  {
    title: '浏览器',
    items: [
      { name: '实例列表', path: '/browser/list', icon: Monitor },
      { name: '自动化脚本', path: '/browser/automation', icon: Aim },
      { name: '内核管理', path: '/browser/cores', icon: Cpu },
      { name: '代理池配置', path: '/browser/proxy-pool', icon: Link },
      { name: '默认书签', path: '/browser/bookmarks', icon: Collection },
      { name: '标签管理', path: '/browser/tags', icon: Tickets },
    ],
  },
  {
    title: '系统',
    items: [
      { name: '系统设置', path: '/settings', icon: Setting },
      { name: '文档中心', path: '/system/docs', icon: Files },
      { name: '日志查看', path: '/browser/logs', icon: Document },
    ],
  },
]
