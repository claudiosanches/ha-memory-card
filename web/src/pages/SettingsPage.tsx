import { Alert, Badge, Button, Card, Group, Loader, Progress, Stack, Table, Text, Title } from '@mantine/core';
import { IconRefresh } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { queryKeys, useIgdbSync, useStatus } from '../api/queries';
import { notifyError } from '../components/notify';
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
