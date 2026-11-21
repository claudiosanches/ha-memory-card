import {
  ActionIcon,
  Anchor,
  Alert,
  Badge,
  Button,
  Code,
  CopyButton,
  Group,
  Loader,
  Select,
  Progress,
  Stack,
  Tabs,
  Text,
  Tooltip,
} from '@mantine/core';
import { IconCheck, IconCopy, IconRefresh } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Link, getRouteApi } from '@tanstack/react-router';
import { ToolTabList } from '../components/ToolTabList';
import { useEffect, useRef, useState } from 'react';
import { ToolSection } from '../components/ToolSection';
import { PageHeader } from '../components/PageHeader';
import classes from './Tools.module.css';
import { useTranslation } from 'react-i18next';
import { browserLanguage, chooseLanguage, type Language, LANGUAGE_NAMES, LANGUAGES, savedLanguage } from '../i18n';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { queryKeys, useCanManage, useIgdbSync, useMe, useStatus, useTitleDifferences } from '../api/queries';
import { Cover } from '../components/Cover';
import { notifyError, notifySuccess } from '../components/notify';
import { themes, useAppearance } from '../styles/themes/AppearanceProvider';
import { ColorSchemePicker } from '../components/ColorSchemePicker';

export const SETTINGS_TABS = ['status', 'igdb', 'appearance'] as const;
export type SettingsTab = (typeof SETTINGS_TABS)[number];

const route = getRouteApi('/settings');

export function SettingsPage() {
  const { t } = useTranslation();
  const status = useStatus();
  const canManage = useCanManage();
  const { themeId, selectTheme } = useAppearance();
  const showIgdb = canManage && status.data?.igdb_configured;
  const { tab = 'status' } = route.useSearch();
  const navigate = route.useNavigate();
  const visibleTabs = SETTINGS_TABS.filter((key) => key !== 'igdb' || showIgdb);

  return (
    <Stack gap={24}>
      <PageHeader title={t('nav.settings')} description={t('settings.intro')} />
      <Tabs
        value={visibleTabs.includes(tab) ? tab : 'status'}
        onChange={(value) => void navigate({ to: '/settings', search: { tab: (value ?? 'status') as SettingsTab } })}
      >
        <ToolTabList label={t('nav.settings')} tabs={visibleTabs.map((key) => ({ value: key, label: t(`settings.${key}`) }))} />
        <Tabs.Panel value="status">
          <Status />
        </Tabs.Panel>
        {showIgdb && (
          <Tabs.Panel value="igdb">
            <Stack gap={24}>
              <IgdbSync />
              <IgdbTitles />
            </Stack>
          </Tabs.Panel>
        )}
        <Tabs.Panel value="appearance">
          <Stack gap={24}>
            <ToolSection
              title={t('settings.theme')}
              action={
                <Select
                  aria-label={t('settings.theme')}
                  value={themeId}
                  allowDeselect={false}
                  onChange={(value) => { if (value) selectTheme(value); }}
                  data={themes.map(({ id }) => ({ value: id, label: t(`settings.themes.${id}`) }))}
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
        { value: 'auto', label: t('settings.languageAuto', { language: LANGUAGE_NAMES[browserLanguage()] }) },
        ...LANGUAGES.map((language) => ({ value: language, label: LANGUAGE_NAMES[language] })),
      ]}
    />
  );
}

function Status() {
  const { t } = useTranslation();
  const status = useStatus();
  return (
    <Stack gap={24}>
      <ToolSection title={t('settings.collection')}>
        {status.isPending && <Loader size="sm" />}
        {status.isError && (
          <Alert color="red" data-testid="status-error">
            {t('settings.loadError')}
          </Alert>
        )}
        {status.data && (
          <dl className={classes.status}>
            <div><dt>{t('settings.games')}</dt><dd>{status.data.games}</dd></div>
            <div><dt>{t('settings.ownedCopies')}</dt><dd>{status.data.owned_copies}</dd></div>
            <div><dt>{t('settings.wishlistItems')}</dt><dd>{status.data.wishlist_items}</dd></div>
            <div><dt>{t('settings.duplicateGames')}</dt><dd>{status.data.duplicate_games}</dd></div>
          </dl>
        )}
      </ToolSection>
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
              <dd data-testid="status-version">{status.data.version}</dd>
            </div>
          )}
          <div>
            <dt>{t('settings.license')}</dt>
            <dd>{t('settings.gpl')}</dd>
          </div>
          <div>
            <dt>{t('settings.author')}</dt>
            <dd>{t('settings.authorName')}</dd>
          </div>
          <div>
            <dt>{t('settings.donations')}</dt>
            <dd><Anchor href="https://github.com/sponsors/claudiosanches" target="_blank" rel="noreferrer">{t('settings.githubSponsors')}</Anchor></dd>
          </div>
        </dl>
      </ToolSection>
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

function IgdbSync() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const sync = useIgdbSync(true);
  const running = sync.data?.running ?? false;

  const start = useMutation({
    mutationFn: () => call(api.POST('/api/igdb/sync')),
    onSuccess: (data) => queryClient.setQueryData(queryKeys.igdbSync, data),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  // Everything IGDB fills shows up in other queries once the run ends.
  const wasRunning = useRef(running);
  useEffect(() => {
    if (wasRunning.current && !running) void queryClient.invalidateQueries();
    wasRunning.current = running;
  }, [running, queryClient]);

  const data = sync.data;
  return (
    <ToolSection title={t('igdb.syncTitle')} description={t('igdb.syncText')}>
      <Group>
        <Button
          leftSection={<IconRefresh size={16} />}
          loading={running || start.isPending}
          onClick={() => start.mutate()}
        >
          {t('igdb.syncButton')}
        </Button>
        <Button component={Link} to="/igdb-matches" variant="default">
          {t('igdb.reviewMatches')}
        </Button>
      </Group>
      {data?.running && (
        <Stack gap={4}>
          <Progress value={data.total ? (data.done / data.total) * 100 : 0} animated />
          <Text size="sm" c="dimmed">
            {t('igdb.syncProgress', { done: data.done, total: data.total })}
          </Text>
        </Stack>
      )}
      {data && !data.running && data.finished_at && (
        <Text size="sm" data-testid="sync-result">
          {t('igdb.syncResult', {
            linked: data.linked,
            updated: data.updated,
            unmatched: data.unmatched,
            failed: data.failed,
          })}
        </Text>
      )}
    </ToolSection>
  );
}

type TitleChoices = { use_igdb?: number[]; keep?: number[] };

function IgdbTitles() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const titles = useTitleDifferences();

  const resolve = useMutation({
    mutationFn: (body: TitleChoices) => call(api.POST('/api/igdb/titles', { body })),
    onSuccess: (left, body) => {
      queryClient.setQueryData(queryKeys.igdbTitles, left);
      if (body.use_igdb?.length) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.games });
        void queryClient.invalidateQueries({ queryKey: ['game'] });
        void queryClient.invalidateQueries({ queryKey: queryKeys.duplicates });
        notifySuccess(t('igdb.titlesSaved'));
      }
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const list = titles.data ?? [];
  return (
    <ToolSection
      title={t('igdb.titlesTitle')}
      description={titles.data ? (list.length ? t('igdb.titlesText', { count: list.length }) : t('igdb.titlesNone')) : undefined}
    >
      {titles.isPending && <Loader size="sm" />}
      {titles.isError && <Text c="red">{errorMessage(t, titles.error)}</Text>}
      {list.length > 1 && (
        <Group>
          <Button
            variant="light"
            disabled={resolve.isPending}
            onClick={() => resolve.mutate({ use_igdb: list.map((d) => d.game_id) })}
          >
            {t('igdb.useAllIgdbTitles')}
          </Button>
        </Group>
      )}
      {list.map((difference) => (
        <Group
          key={difference.game_id}
          className={classes.titleRow}
          justify="space-between"
          gap="sm"
          data-testid={`title-${difference.game_id}`}
        >
          <Group gap="sm" wrap="nowrap" miw={0} style={{ flex: '1 1 260px' }}>
            <Cover path={difference.cover_path} title={difference.title} width={36} />
            <Stack gap={0} miw={0}>
              <Text size="sm" fw={600}>
                {difference.title}
              </Text>
              <Text size="sm" c="dimmed">
                {t('igdb.titleOnIgdb', { title: difference.igdb_title })}
              </Text>
            </Stack>
          </Group>
          <Group className={classes.titleActions} gap="xs">
            <Button
              size="xs"
              variant="default"
              disabled={resolve.isPending}
              onClick={() => resolve.mutate({ keep: [difference.game_id] })}
            >
              {t('igdb.keepTitle')}
            </Button>
            <Button
              size="xs"
              disabled={resolve.isPending}
              onClick={() => resolve.mutate({ use_igdb: [difference.game_id] })}
            >
              {t('igdb.useIgdbTitle')}
            </Button>
          </Group>
        </Group>
      ))}
    </ToolSection>
  );
}
