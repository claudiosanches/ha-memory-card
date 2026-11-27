import { useData } from 'vitepress';
import { computed } from 'vue';

export const LOCALE_PREFIX = {
  en: '',
  'pt-BR': '/pt-br',
  es: '/es',
};

const COPY = {
  en: {
    prefix: '',
    hero: {
      lightAlt: 'The library in Memory Card, light theme',
      darkAlt: 'The library in Memory Card, dark theme',
      trademark: 'Game titles and cover art belong to their respective owners.',
    },
    themes: {
      heading: '12 built-in themes',
      subtitle: 'Five quiet palettes and seven expressive themes, each in light and dark.',
      browseAll: 'Browse all 12 themes in light & dark',
      dark: 'Dark',
      light: 'Light',
      items: {
        retro: { name: 'Retro', alt: 'Retro theme in dark mode' },
        pastelPlush: { name: 'Pastel Plush', alt: 'Pastel Plush theme in light mode' },
        cyberpunk: { name: 'Cyberpunk', alt: 'Cyberpunk theme in dark mode' },
      },
    },
    install: {
      heading: 'Install in Home Assistant',
      requires: 'Requires Home Assistant 2026.11 or newer',
      badgeAlt:
        'Open your Home Assistant instance and show the add app repository dialog with a specific repository URL pre-filled.',
      step1Title: 'Add the repository',
      step1Before: 'Select the button above, or go to ',
      step1Path: 'Settings > Apps > Install app',
      step1Middle: ', open the ⋮ menu, choose ',
      step1Repos: 'Repositories',
      step1After: ' and paste:',
      copy: 'Copy',
      copied: 'Copied',
      step2Title: 'Install and start',
      step2Before: 'Install ',
      step2App: 'Memory Card',
      step2After: ' and start it. It appears in your Home Assistant sidebar with no separate login required.',
      gettingStarted: 'Follow the getting started guide',
      supportHeading: 'Free and open source',
      supportText: 'If Memory Card is useful to your household, you can support ongoing development.',
      githubSponsors: 'GitHub Sponsors',
      kofi: 'Support on Ko-fi',
    },
    zoom: {
      prev: 'Previous image',
      next: 'Next image',
      of: (index, total) => `${index} of ${total}`,
    },
  },
  'pt-BR': {
    prefix: '/pt-br',
    hero: {
      lightAlt: 'A biblioteca no Memory Card, tema claro',
      darkAlt: 'A biblioteca no Memory Card, tema escuro',
      trademark: 'Títulos de jogos e capas pertencem aos seus respectivos proprietários.',
    },
    themes: {
      heading: '12 temas integrados',
      subtitle: 'Cinco paletas discretas e sete temas estilizados, cada um em claro e escuro.',
      browseAll: 'Ver todos os 12 temas em claro e escuro',
      dark: 'Escuro',
      light: 'Claro',
      items: {
        retro: { name: 'Retrô', alt: 'Tema Retrô no modo escuro' },
        pastelPlush: { name: 'Pelúcia pastel', alt: 'Tema Pelúcia pastel no modo claro' },
        cyberpunk: { name: 'Cyberpunk', alt: 'Tema Cyberpunk no modo escuro' },
      },
    },
    install: {
      heading: 'Instalar no Home Assistant',
      requires: 'Requer Home Assistant 2026.11 ou mais recente',
      badgeAlt:
        'Abrir sua instância do Home Assistant na janela de adicionar repositório de aplicativos com a URL do repositório preenchida.',
      step1Title: 'Adicione o repositório',
      step1Before: 'Clique no botão acima ou vá em ',
      step1Path: 'Configurações > Aplicativos > Instalar aplicativo',
      step1Middle: ', abra o menu ⋮, escolha ',
      step1Repos: 'Repositórios',
      step1After: ' e cole:',
      copy: 'Copiar',
      copied: 'Copiado',
      step2Title: 'Instale e inicie',
      step2Before: 'Instale o ',
      step2App: 'Memory Card',
      step2After: ' e inicie o aplicativo. Ele aparece na barra lateral do Home Assistant sem precisar de outro login.',
      gettingStarted: 'Seguir o guia de primeiros passos',
      supportHeading: 'Gratuito e de código aberto',
      supportText: 'Se o Memory Card é útil para sua casa, você pode apoiar o desenvolvimento contínuo.',
      githubSponsors: 'GitHub Sponsors',
      kofi: 'Apoiar no Ko-fi',
    },
    zoom: {
      prev: 'Imagem anterior',
      next: 'Próxima imagem',
      of: (index, total) => `${index} de ${total}`,
    },
  },
  es: {
    prefix: '/es',
    hero: {
      lightAlt: 'La biblioteca en Memory Card, tema claro',
      darkAlt: 'La biblioteca en Memory Card, tema oscuro',
      trademark: 'Los títulos de juegos y las portadas pertenecen a sus respectivos propietarios.',
    },
    themes: {
      heading: '12 temas integrados',
      subtitle: 'Cinco paletas sobrias y siete temas estilizados, cada uno en claro y oscuro.',
      browseAll: 'Ver los 12 temas en claro y oscuro',
      dark: 'Oscuro',
      light: 'Claro',
      items: {
        retro: { name: 'Retro', alt: 'Tema Retro en modo oscuro' },
        pastelPlush: { name: 'Peluche pastel', alt: 'Tema Peluche pastel en modo claro' },
        cyberpunk: { name: 'Cyberpunk', alt: 'Tema Cyberpunk en modo oscuro' },
      },
    },
    install: {
      heading: 'Instalar en Home Assistant',
      requires: 'Requiere Home Assistant 2026.11 o posterior',
      badgeAlt:
        'Abrir tu instancia de Home Assistant en el diálogo para añadir repositorios de aplicaciones con la URL del repositorio rellenada.',
      step1Title: 'Añade el repositorio',
      step1Before: 'Selecciona el botón de arriba o ve a ',
      step1Path: 'Ajustes > Aplicaciones > Instalar aplicación',
      step1Middle: ', abre el menú ⋮, elige ',
      step1Repos: 'Repositorios',
      step1After: ' y pega:',
      copy: 'Copiar',
      copied: 'Copiado',
      step2Title: 'Instala e inicia',
      step2Before: 'Instala ',
      step2App: 'Memory Card',
      step2After: ' e inícialo. Aparece en la barra lateral de Home Assistant sin necesidad de iniciar sesión aparte.',
      gettingStarted: 'Seguir la guía de primeros pasos',
      supportHeading: 'Gratuito y de código abierto',
      supportText: 'Si Memory Card le resulta útil a tu casa, puedes apoyar el desarrollo continuo.',
      githubSponsors: 'GitHub Sponsors',
      kofi: 'Apoyar en Ko-fi',
    },
    zoom: {
      prev: 'Imagen anterior',
      next: 'Imagen siguiente',
      of: (index, total) => `${index} de ${total}`,
    },
  },
};

export function getCopy(lang) {
  return COPY[lang] ?? COPY.en;
}

export function useThemeCopy() {
  const { lang } = useData();
  return computed(() => getCopy(lang.value));
}
