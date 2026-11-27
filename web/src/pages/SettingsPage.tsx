import {
  ActionIcon,
  Anchor,
  Alert,
  Badge,
  Code,
  CopyButton,
  Divider,
  Group,
  Loader,
  Select,
  Stack,
  Tabs,
  Text,
  Tooltip,
} from '@mantine/core';
import { IconCheck, IconCopy } from '@tabler/icons-react';
import { getRouteApi } from '@tanstack/react-router';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useMe, useStatus } from '../api/queries';
import { ColorSchemePicker } from '../components/ColorSchemePicker';
import { DiagnosticTerminalModal } from '../components/DiagnosticTerminalModal';
import { PageHeader } from '../components/PageHeader';
import { ToolSection } from '../components/ToolSection';
import { ToolTabList } from '../components/ToolTabList';
import { automaticLanguage, chooseLanguage, type Language, LANGUAGE_NAMES, LANGUAGES, savedLanguage } from '../i18n';
import { useAppearance } from '../styles/themes/AppearanceProvider';
import classes from './Tools.module.css';

export const SETTINGS_TABS = ['appearance', 'status'] as const;
export type SettingsTab = (typeof SETTINGS_TABS)[number];

const route = getRouteApi('/settings');

export function SettingsPage() {
  const { t } = useTranslation();
  const me = useMe();
  const { themeId, selectTheme, availableThemes } = useAppearance();
  const defaultTab: SettingsTab = me.data && !me.data.managers_configured ? 'status' : 'appearance';
  const { tab } = route.useSearch();
  const activeTab = tab && SETTINGS_TABS.includes(tab) ? tab : defaultTab;
  const navigate = route.useNavigate();

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.settings')} description={t('settings.intro')} />
      <Tabs
        value={activeTab}
        onChange={(value) => void navigate({ to: '/settings', search: { tab: (value ?? defaultTab) as SettingsTab } })}
      >
        <ToolTabList label={t('nav.settings')} tabs={SETTINGS_TABS.map((key) => ({ value: key, label: t(`settings.${key}`) }))} />
        <Tabs.Panel value="appearance">
          <Stack gap="card">
            <ToolSection
              title={t('settings.theme')}
              action={
                <Select
                  aria-label={t('settings.theme')}
                  value={themeId}
                  allowDeselect={false}
                  onChange={(value) => { if (value) selectTheme(value); }}
                  data={availableThemes.map(({ id }) => ({ value: id, label: t(`settings.themes.${id}`) }))}
                />
              }
            />
            <ToolSection
              title={t('settings.colorScheme')}
              description={t('settings.colorSchemeDescription')}
              action={<ColorSchemePicker />}
            />
            <ToolSection
              title={t('settings.language')}
              description={t('settings.languageDescription')}
              action={<LanguagePicker />}
            />
          </Stack>
        </Tabs.Panel>
        <Tabs.Panel value="status">
          <Status />
        </Tabs.Panel>
      </Tabs>
    </Stack>
  );
}

function LanguagePicker() {
  const { t } = useTranslation();
  const [choice, setChoice] = useState<Language | null>(() => savedLanguage());
  return (
    <Select
      aria-label={t('settings.language')}
      value={choice ?? 'auto'}
      allowDeselect={false}
      onChange={(value) => {
        const next = LANGUAGES.find((language) => language === value) ?? null;
        setChoice(next);
        chooseLanguage(next);
      }}
      data={[
        { value: 'auto', label: t('settings.languageAuto', { language: LANGUAGE_NAMES[automaticLanguage()] }) },
        ...LANGUAGES.map((language) => ({ value: language, label: LANGUAGE_NAMES[language] })),
      ]}
    />
  );
}

const DOCS_URL: Record<Language, string> = {
  en: 'https://ha-memory-card.pages.dev/docs/',
  'pt-BR': 'https://ha-memory-card.pages.dev/pt-br/docs/',
  es: 'https://ha-memory-card.pages.dev/es/docs/',
};

function Status() {
  const { t, i18n } = useTranslation();
  const status = useStatus();
  const docsUrl = DOCS_URL[i18n.language as Language] ?? DOCS_URL.en;
  const [terminalOpen, setTerminalOpen] = useState(false);
  const versionTaps = useRef<{ count: number; lastAt: number }>({ count: 0, lastAt: 0 });

  const tapVersion = () => {
    const now = Date.now();
    const count = now - versionTaps.current.lastAt < 2500 ? versionTaps.current.count + 1 : 1;
    versionTaps.current = { count, lastAt: now };
    if (count >= 3) {
      versionTaps.current = { count: 0, lastAt: 0 };
      setTerminalOpen(true);
    }
  };

  return (
    <Stack gap="card">
      {status.isPending && <Loader size="sm" />}
      {status.isError && (
        <Alert color="red" data-testid="status-error">
          {t('settings.loadError')}
        </Alert>
      )}
      {status.data && (
        <ToolSection title={t('settings.integrations')}>
          <dl className={classes.integrations}>
            <div>
              <dt>{t('settings.igdb')}</dt>
              <dd>
                {status.data.igdb_configured ? (
                  <Badge color="green">{t('settings.igdbConfigured')}</Badge>
                ) : (
                  <Group>
                    <Text size="sm" c="dimmed" data-testid="igdb-missing">
                      {status.data.external_services ? t('settings.igdbMissing') : t('settings.igdbOffline')}
                    </Text>
                  </Group>
                )}
              </dd>
            </div>
            <div>
              <dt>{t('settings.sensors')}</dt>
              <dd data-testid="sensors-status">
                {status.data.sensors === 'connected' ? (
                  <Badge color="green">{t('settings.sensorsStatus.connected')}</Badge>
                ) : (
                  <Text size="sm" c="dimmed">
                    {t(`settings.sensorsStatus.${status.data.sensors}`)}
                  </Text>
                )}
              </dd>
            </div>
            <div>
              <dt>{t('settings.tags')}</dt>
              <dd data-testid="tags-status">
                {status.data.tags === 'connected' ? (
                  <Badge color="green">{t('settings.tagsStatus.connected')}</Badge>
                ) : (
                  <Text size="sm" c="dimmed">
                    {t(`settings.tagsStatus.${status.data.tags}`)}
                  </Text>
                )}
              </dd>
            </div>
          </dl>
        </ToolSection>
      )}
      <Access />
      <ToolSection title={t('settings.about')}>
        <dl className={classes.integrations}>
          {status.data && (
            <div>
              <dt>{t('settings.version')}</dt>
              <dd data-testid="status-version" onClick={tapVersion}>
                {status.data.version}
              </dd>
            </div>
          )}
          <div>
            <dt>{t('settings.license')}</dt>
            <dd>{t('settings.licenseName')}</dd>
          </div>
          <div>
            <dt>{t('settings.author')}</dt>
            <dd>{t('settings.authorName')}</dd>
          </div>
          <div>
            <dt>{t('settings.help')}</dt>
            <dd>
              <Group gap="sm">
                <Anchor href={docsUrl} target="_blank" rel="noreferrer">
                  {t('settings.documentation')}
                </Anchor>
                <Divider orientation="vertical" />
                <Anchor
                  href="https://github.com/claudiosanches/ha-memory-card/issues/new?template=support.yml"
                  target="_blank"
                  rel="noreferrer"
                >
                  {t('settings.getSupport')}
                </Anchor>
              </Group>
            </dd>
          </div>
          <div>
            <dt>{t('settings.donations')}</dt>
            <dd>
              <Group gap="sm">
                <Anchor href="https://github.com/sponsors/claudiosanches" target="_blank" rel="noreferrer">
                  {t('settings.githubSponsors')}
                </Anchor>
                <Divider orientation="vertical" />
                <Anchor href="https://ko-fi.com/claudiosanches" target="_blank" rel="noreferrer">
                  {t('settings.kofi')}
                </Anchor>
              </Group>
            </dd>
          </div>
        </dl>
      </ToolSection>
      {status.data && terminalOpen && (
        <DiagnosticTerminalModal
          opened={terminalOpen}
          version={status.data.version}
          onClose={() => setTerminalOpen(false)}
        />
      )}
    </Stack>
  );
}

/** Who Home Assistant says is signed in, with the ID to put in the managers option. */
function Access() {
  const { t } = useTranslation();
  const me = useMe().data;
  if (!me) return null;

  const description = !me.managers_configured
    ? t('settings.accessNoManagers')
    : me.role === 'manager'
      ? t('settings.accessManager')
      : me.can_track_progress
        ? t('settings.accessViewerProgress')
        : t('settings.accessViewer');
  const user = me.user;
  return (
    <ToolSection title={t('settings.access')} description={description}>
      <dl className={classes.status}>
        <div>
          <dt>{t('settings.signedInAs')}</dt>
          <dd>{user.display_name ?? user.name ?? user.id}</dd>
        </div>
        <div>
          <dt>{t('settings.role')}</dt>
          <dd data-testid="access-role">{t(`settings.roles.${me.role}`)}</dd>
        </div>
        <div>
          <dt>{t('settings.userId')}</dt>
          <dd className={classes.userId}>
            <Code data-testid="user-id">{user.id}</Code>
            <CopyButton value={user.id}>
              {({ copied, copy }) => (
                <Tooltip label={copied ? t('settings.copied') : t('settings.copyUserId')}>
                  <ActionIcon variant="subtle" aria-label={t('settings.copyUserId')} onClick={copy}>
                    {copied ? <IconCheck size={16} /> : <IconCopy size={16} />}
                  </ActionIcon>
                </Tooltip>
              )}
            </CopyButton>
          </dd>
        </div>
      </dl>
    </ToolSection>
  );
}
