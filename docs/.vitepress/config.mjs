// The docs site, built with `make docs`. docs/index.md is the landing page,
// docs/about.md the About page, docs/README.md the guide's index, and the
// guide's pages sit under /docs/.
import { readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// Developer pages stay on GitHub, linked from the root README.
const developer = ['development.md', 'local-home-assistant.md', 'releasing.md'];
const site = ['index.md', 'about.md'];
const guide = readdirSync(fileURLToPath(new URL('..', import.meta.url))).filter(
  (file) => file.endsWith('.md') && !site.includes(file) && !developer.includes(file),
);

const page = (text, file) => ({ text, link: `/docs/${file}` });

/** @type {import('vitepress').UserConfig} */
export default {
  title: 'Memory Card',
  description: 'A video game collection manager that runs as a Home Assistant app.',
  lang: 'en',
  cleanUrls: true,
  markdown: { image: { lazyLoading: true } },
  srcExclude: developer,
  rewrites: Object.fromEntries(guide.map((file) => [file, file === 'README.md' ? 'docs/index.md' : `docs/${file}`])),
  head: [['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }]],
  sitemap: { hostname: 'https://ha-memory-card.pages.dev' },
  themeConfig: {
    logo: '/favicon.svg',
    nav: [
      { text: 'Guide', link: '/docs/' },
      { text: 'Get started', link: '/docs/getting-started' },
      { text: 'About', link: '/about' },
    ],
    sidebar: {
      '/docs/': [
        { text: 'Overview', link: '/docs/' },
        { text: 'Start here', items: [page('Getting started', 'getting-started')] },
        {
          text: 'Pages',
          items: [
            page('Library', 'library'),
            page('Game page', 'game'),
            page('Backlog', 'backlog'),
            page('Wishlist', 'wishlist'),
            page('Hardware', 'hardware'),
            page('Stats', 'stats'),
            page('Household', 'household'),
            page('Manage', 'manage'),
            page('Cleanup', 'cleanup'),
            page('Import & export', 'import-export'),
            page('Settings', 'settings'),
          ],
        },
        {
          text: 'Features',
          items: [
            page('IGDB', 'igdb'),
            page('Barcodes', 'barcodes'),
            page('NFC tags', 'nfc-tags'),
            page('Home Assistant sensors', 'sensors'),
            page('Memory Card integration', 'integration'),
            page('Themes', 'themes'),
            page('Data and backups', 'data'),
          ],
        },
      ],
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/claudiosanches/ha-memory-card' }],
    search: { provider: 'local' },
    footer: {
      message: 'Released under the Apache License 2.0.',
      copyright:
        'Game titles, cover art and other trademarks belong to their respective owners. Memory Card is not affiliated with or endorsed by them.',
    },
  },
};
