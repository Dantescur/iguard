import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'iguard',
  description: 'Graphical Polkit authentication agent for Linux desktop sessions',
  base: '/iguard/',
  head: [
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', {
      rel: 'stylesheet',
      href: 'https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,400..800&family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap'
    }],
    ['meta', { name: 'theme-color', content: '#0b1017' }],
    ['link', { rel: 'icon', type: 'image/png', href: '/iguard/logo.png' }]
  ],
  themeConfig: {
    logo: '/logo.png',
    nav: [
      { text: 'Guide', link: '/guide/installation' },
      { text: 'Configuration', link: '/guide/configuration' },
      { text: 'How it works', link: '/guide/how-it-works' }
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Installation', link: '/guide/installation' },
          { text: 'Configuration', link: '/guide/configuration' },
          { text: 'How it works', link: '/guide/how-it-works' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/Dantescur/iguard' }
    ],
    search: {
      provider: 'local'
    },
    editLink: {
      pattern: 'https://github.com/Dantescur/iguard/edit/main/docs/:path',
      text: 'Edit this page'
    },
    outline: {
      level: [2, 3],
      label: 'On this page'
    },
    footer: {
      message: 'Source MIT · release binaries GPL-3.0-or-later',
      copyright: 'Copyright © 2026 iguard contributors'
    }
  }
})
