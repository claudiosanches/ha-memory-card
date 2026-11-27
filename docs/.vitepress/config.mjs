// The docs site, built with `make docs`. docs/index.md is the landing page,
// docs/about.md the About page, docs/README.md the guide's index, and the
// guide's pages sit under /docs/. Translated locales live under docs/<locale>/.
import { existsSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const docsDir = fileURLToPath(new URL('..', import.meta.url));

// Developer pages stay on GitHub, linked from the root README.
const developer = ['development.md', 'local-home-assistant.md', 'releasing.md'];
const site = ['index.md', 'about.md'];
const translatedLocales = ['pt-br', 'es'];

const guideFiles = (dir) =>
  existsSync(dir)
    ? readdirSync(dir).filter((file) => file.endsWith('.md') && !site.includes(file) && !developer.includes(file))
    : [];

const rewrites = Object.fromEntries([
  ...guideFiles(docsDir).map((file) => [file, file === 'README.md' ? 'docs/index.md' : `docs/${file}`]),
  ...translatedLocales.flatMap((locale) =>
    guideFiles(`${docsDir}/${locale}`).map((file) => [
      `${locale}/${file}`,
      file === 'README.md' ? `${locale}/docs/index.md` : `${locale}/docs/${file}`,
    ]),
  ),
]);

const page = (prefix) => (text, file) => ({ text, link: `${prefix}/docs/${file}` });

function buildSidebar(prefix, labels) {
  const p = page(prefix);
  return {
    [`${prefix}/docs/`]: [
      { text: labels.overview, link: `${prefix}/docs/` },
      { text: labels.startHere, items: [p(labels.gettingStarted, 'getting-started')] },
      {
        text: labels.pages,
        items: [
          p(labels.library, 'library'),
          p(labels.game, 'game'),
          p(labels.backlog, 'backlog'),
          p(labels.wishlist, 'wishlist'),
          p(labels.hardware, 'hardware'),
          p(labels.stats, 'stats'),
          p(labels.household, 'household'),
          p(labels.manage, 'manage'),
          p(labels.maintenance, 'maintenance'),
          p(labels.importExport, 'import-export'),
          p(labels.settings, 'settings'),
        ],
      },
      {
        text: labels.features,
        items: [
          p(labels.igdb, 'igdb'),
          p(labels.barcodes, 'barcodes'),
          p(labels.nfcTags, 'nfc-tags'),
          p(labels.sensors, 'sensors'),
          p(labels.integration, 'integration'),
          p(labels.themes, 'themes'),
          p(labels.data, 'data'),
        ],
      },
    ],
  };
}

/** @type {import('vitepress').UserConfig} */
export default {
  title: 'Memory Card',
  cleanUrls: true,
  markdown: { image: { lazyLoading: true } },
  srcExclude: developer,
  rewrites,
  head: [['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }]],
  sitemap: { hostname: 'https://ha-memory-card.pages.dev' },
  themeConfig: {
    logo: '/favicon.svg',
    socialLinks: [{ icon: 'github', link: 'https://github.com/claudiosanches/ha-memory-card' }],
    search: {
      provider: 'local',
      options: {
        locales: {
          'pt-br': {
            translations: {
              button: { buttonText: 'Buscar', buttonAriaLabel: 'Buscar' },
              modal: {
                displayDetails: 'Mostrar lista detalhada',
                resetButtonTitle: 'Limpar busca',
                backButtonTitle: 'Fechar busca',
                noResultsText: 'Nenhum resultado para',
                footer: {
                  selectText: 'para selecionar',
                  selectKeyAriaLabel: 'enter',
                  navigateText: 'para navegar',
                  navigateUpKeyAriaLabel: 'seta para cima',
                  navigateDownKeyAriaLabel: 'seta para baixo',
                  closeText: 'para fechar',
                  closeKeyAriaLabel: 'escape',
                },
              },
            },
          },
          es: {
            translations: {
              button: { buttonText: 'Buscar', buttonAriaLabel: 'Buscar' },
              modal: {
                displayDetails: 'Mostrar lista detallada',
                resetButtonTitle: 'Limpiar búsqueda',
                backButtonTitle: 'Cerrar búsqueda',
                noResultsText: 'Sin resultados para',
                footer: {
                  selectText: 'para seleccionar',
                  selectKeyAriaLabel: 'intro',
                  navigateText: 'para navegar',
                  navigateUpKeyAriaLabel: 'flecha arriba',
                  navigateDownKeyAriaLabel: 'flecha abajo',
                  closeText: 'para cerrar',
                  closeKeyAriaLabel: 'escape',
                },
              },
            },
          },
        },
      },
    },
  },
  locales: {
    root: {
      label: 'English',
      lang: 'en',
      description: 'A video game collection manager that runs as a Home Assistant app.',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/docs/' },
          { text: 'Get started', link: '/docs/getting-started' },
          { text: 'About', link: '/about' },
        ],
        sidebar: buildSidebar('', {
          overview: 'Overview',
          startHere: 'Start here',
          gettingStarted: 'Getting started',
          pages: 'Pages',
          library: 'Library',
          game: 'Game page',
          backlog: 'Backlog',
          wishlist: 'Wishlist',
          hardware: 'Hardware',
          stats: 'Stats',
          household: 'Household',
          manage: 'Manage',
          maintenance: 'Maintenance',
          importExport: 'Import & export',
          settings: 'Settings',
          features: 'Features',
          igdb: 'IGDB',
          barcodes: 'Barcodes',
          nfcTags: 'NFC tags',
          sensors: 'Home Assistant sensors',
          integration: 'Memory Card integration',
          themes: 'Themes',
          data: 'Data and backups',
        }),
        footer: {
          message: 'Released under the Apache License 2.0.',
          copyright:
            'Game titles, cover art and other trademarks belong to their respective owners. Memory Card is not affiliated with or endorsed by them.',
        },
      },
    },
    'pt-br': {
      label: 'Português (Brasil)',
      lang: 'pt-BR',
      link: '/pt-br/',
      description: 'Um gerenciador de coleção de jogos que roda como aplicativo no Home Assistant.',
      themeConfig: {
        nav: [
          { text: 'Guia', link: '/pt-br/docs/' },
          { text: 'Primeiros passos', link: '/pt-br/docs/getting-started' },
          { text: 'Sobre', link: '/pt-br/about' },
        ],
        sidebar: buildSidebar('/pt-br', {
          overview: 'Visão geral',
          startHere: 'Comece aqui',
          gettingStarted: 'Primeiros passos',
          pages: 'Páginas',
          library: 'Biblioteca',
          game: 'Página do jogo',
          backlog: 'Backlog',
          wishlist: 'Lista de desejos',
          hardware: 'Hardware',
          stats: 'Estatísticas',
          household: 'Casa',
          manage: 'Gerenciar',
          maintenance: 'Manutenção',
          importExport: 'Importar / exportar',
          settings: 'Configurações',
          features: 'Recursos',
          igdb: 'IGDB',
          barcodes: 'Códigos de barras',
          nfcTags: 'Tags NFC',
          sensors: 'Sensores do Home Assistant',
          integration: 'Integração Memory Card',
          themes: 'Temas',
          data: 'Dados e backups',
        }),
        outline: { label: 'Nesta página' },
        docFooter: { prev: 'Página anterior', next: 'Próxima página' },
        returnToTopLabel: 'Voltar ao topo',
        sidebarMenuLabel: 'Menu',
        darkModeSwitchLabel: 'Aparência',
        lightModeSwitchTitle: 'Mudar para o tema claro',
        darkModeSwitchTitle: 'Mudar para o tema escuro',
        langMenuLabel: 'Mudar idioma',
        notFound: {
          title: 'PÁGINA NÃO ENCONTRADA',
          quote: 'Não encontramos o endereço que você tentou abrir.',
          linkLabel: 'Ir para a página inicial',
          linkText: 'Voltar para o início',
        },
        footer: {
          message: 'Distribuído sob a licença Apache 2.0.',
          copyright:
            'Títulos de jogos, capas e outras marcas pertencem aos seus respectivos proprietários. O Memory Card não é afiliado nem endossado por eles.',
        },
      },
    },
    es: {
      label: 'Español',
      lang: 'es',
      link: '/es/',
      description: 'Un gestor de colección de videojuegos que funciona como aplicación de Home Assistant.',
      themeConfig: {
        nav: [
          { text: 'Guía', link: '/es/docs/' },
          { text: 'Primeros pasos', link: '/es/docs/getting-started' },
          { text: 'Acerca de', link: '/es/about' },
        ],
        sidebar: buildSidebar('/es', {
          overview: 'Resumen',
          startHere: 'Empieza aquí',
          gettingStarted: 'Primeros pasos',
          pages: 'Páginas',
          library: 'Biblioteca',
          game: 'Página del juego',
          backlog: 'Pendientes',
          wishlist: 'Lista de deseos',
          hardware: 'Hardware',
          stats: 'Estadísticas',
          household: 'Hogar',
          manage: 'Gestionar',
          maintenance: 'Mantenimiento',
          importExport: 'Importar / exportar',
          settings: 'Ajustes',
          features: 'Funciones',
          igdb: 'IGDB',
          barcodes: 'Códigos de barras',
          nfcTags: 'Etiquetas NFC',
          sensors: 'Sensores de Home Assistant',
          integration: 'Integración Memory Card',
          themes: 'Temas',
          data: 'Datos y copias de seguridad',
        }),
        outline: { label: 'En esta página' },
        docFooter: { prev: 'Página anterior', next: 'Página siguiente' },
        returnToTopLabel: 'Volver arriba',
        sidebarMenuLabel: 'Menú',
        darkModeSwitchLabel: 'Apariencia',
        lightModeSwitchTitle: 'Cambiar al tema claro',
        darkModeSwitchTitle: 'Cambiar al tema oscuro',
        langMenuLabel: 'Cambiar idioma',
        notFound: {
          title: 'PÁGINA NO ENCONTRADA',
          quote: 'No encontramos la dirección que intentaste abrir.',
          linkLabel: 'Ir a la página principal',
          linkText: 'Volver al inicio',
        },
        footer: {
          message: 'Publicado bajo la licencia Apache 2.0.',
          copyright:
            'Los títulos de juegos, las portadas y otras marcas pertenecen a sus respectivos propietarios. Memory Card no está afiliado ni respaldado por ellos.',
        },
      },
    },
  },
};
