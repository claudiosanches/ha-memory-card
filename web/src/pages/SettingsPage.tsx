import { Alert, Badge, Card, Group, Loader, Stack, Table, Text, Title } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';

export function SettingsPage() {
  const { t } = useTranslation();
  const status = useQuery({
    queryKey: ['status'],
    queryFn: async () => {
      const { data, error } = await api.GET('/api/status');
      if (error || !data) throw new Error(error?.code ?? 'network');
      return data;
    },
  });

  return (
    <Stack>
      <Title order={2}>{t('nav.settings')}</Title>
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
    </Stack>
  );
}
