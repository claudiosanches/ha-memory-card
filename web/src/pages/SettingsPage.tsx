import { Alert, Badge, Button, Card, Group, Loader, Progress, Stack, Table, Text, Title } from '@mantine/core';
import { IconRefresh } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';
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
    <Stack>
      <Title order={2}>{t('nav.settings')}</Title>
      <Card withBorder>
        <Group justify="space-between">
          <Stack gap={4}>
            <Title order={4}>{t('settings.appearance')}</Title>
            <Text size="sm" c="dimmed">{t('settings.appearanceDescription')}</Text>
          </Stack>
          <ThemePicker />
        </Group>
      </Card>
      <Card withBorder>
        <Title order={4} mb="sm">
          {t('settings.status')}
        </Title>
        {status.isPending && <Loader size="sm" />}
        {status.isError && (
          <Alert color="red" data-testid="status-error">
            {t('settings.loadError')}
          </Alert>
        )}
        {status.data && (
          <Table>
            <Table.Tbody>
              <Table.Tr>
                <Table.Th>{t('settings.version')}</Table.Th>
                <Table.Td data-testid="status-version">{status.data.version}</Table.Td>
              </Table.Tr>
              <Table.Tr>
                <Table.Th>{t('settings.games')}</Table.Th>
                <Table.Td>{status.data.games}</Table.Td>
              </Table.Tr>
              <Table.Tr>
                <Table.Th>{t('settings.ownedCopies')}</Table.Th>
                <Table.Td>{status.data.owned_copies}</Table.Td>
              </Table.Tr>
              <Table.Tr>
                <Table.Th>{t('settings.igdb')}</Table.Th>
                <Table.Td>
                  {status.data.igdb_configured ? (
                    <Badge color="green">{t('settings.igdbConfigured')}</Badge>
                  ) : (
                    <Group>
                      <Text size="sm" c="dimmed" data-testid="igdb-missing">
                        {t('settings.igdbMissing')}
                      </Text>
                    </Group>
                  )}
                </Table.Td>
              </Table.Tr>
            </Table.Tbody>
          </Table>
        )}
      </Card>
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
    <Card withBorder>
      <Stack gap="sm">
        <Title order={4}>{t('igdb.syncTitle')}</Title>
        <Text size="sm" c="dimmed">
          {t('igdb.syncText')}
        </Text>
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
      </Stack>
    </Card>
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
    <Card withBorder>
      <Stack gap="sm">
        <Title order={4}>{t('igdb.titlesTitle')}</Title>
        {titles.isPending && <Loader size="sm" />}
        {titles.isError && <Text c="red">{errorMessage(t, titles.error)}</Text>}
        {titles.data && (
          <Text size="sm" c="dimmed">
            {list.length ? t('igdb.titlesText', { count: list.length }) : t('igdb.titlesNone')}
          </Text>
        )}
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
            <Group gap="xs" wrap="nowrap">
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
      </Stack>
    </Card>
  );
}
