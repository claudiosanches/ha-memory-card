import { Alert, Badge, Button, Group, Loader, Progress, Stack, Text } from '@mantine/core';
import { IconRefresh } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';
import { ToolSection } from '../components/ToolSection';
import { PageHeader } from '../components/PageHeader';
import classes from './Tools.module.css';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { queryKeys, useIgdbSync, useStatus, useTitleDifferences } from '../api/queries';
import { Cover } from '../components/Cover';
import { notifyError, notifySuccess } from '../components/notify';
import { ThemePicker } from '../components/ThemePicker';

export function SettingsPage() {
  const { t } = useTranslation();
  const status = useStatus();

  return (
    <Stack gap={24}>
      <PageHeader title={t('nav.settings')} description={t('settings.intro')} />
      <ToolSection
        title={t('settings.appearance')}
        description={t('settings.appearanceDescription')}
        action={<ThemePicker />}
      />
      <ToolSection title={t('settings.status')}>
        {status.isPending && <Loader size="sm" />}
        {status.isError && (
          <Alert color="red" data-testid="status-error">
            {t('settings.loadError')}
          </Alert>
        )}
        {status.data && (
          <dl className={classes.status}>
            <div><dt>{t('settings.version')}</dt><dd data-testid="status-version">{status.data.version}</dd></div>
            <div><dt>{t('settings.games')}</dt><dd>{status.data.games}</dd></div>
            <div><dt>{t('settings.ownedCopies')}</dt><dd>{status.data.owned_copies}</dd></div>
            <div>
              <dt>{t('settings.igdb')}</dt>
              <dd>
                {status.data.igdb_configured ? (
                  <Badge color="green">{t('settings.igdbConfigured')}</Badge>
                ) : (
                  <Group>
                    <Text size="sm" c="dimmed" data-testid="igdb-missing">
                      {t('settings.igdbMissing')}
                    </Text>
                  </Group>
                )}
              </dd>
            </div>
          </dl>
        )}
      </ToolSection>
      {status.data?.igdb_configured && <IgdbSync />}
      {status.data?.igdb_configured && <IgdbTitles />}
    </Stack>
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
