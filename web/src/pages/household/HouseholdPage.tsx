import { Badge, Card, Group, Loader, SegmentedControl, SimpleGrid, Stack, Text } from '@mantine/core';
import { useLocalStorage } from '@mantine/hooks';
import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useMe } from '../../api/queries';
import type { components } from '../../api/schema';
import { PageHeader } from '../../components/PageHeader';
import { playTime } from '../stats/StatsPage';
import classes from './HouseholdPage.module.css';

type Period = 'month' | 'year' | 'all';
type PersonTotals = components['schemas']['PersonTotals'];

const PERIODS: Period[] = ['month', 'year', 'all'];

function useHousehold(period: Period) {
  return useQuery({
    queryKey: [...queryKeys.household, period],
    queryFn: () => call(api.GET('/api/household', { params: { query: { period } } })),
  });
}

/** Everyone's play side by side, the most play time first. */
export function HouseholdPage() {
  const { t } = useTranslation();
  const [stored, setPeriod] = useLocalStorage<Period>({ key: 'memory-card-household-period', defaultValue: 'month' });
  const period = PERIODS.includes(stored) ? stored : 'month';
  const household = useHousehold(period);
  const me = useMe().data;

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.household')} description={t('household.intro')} />
      <SegmentedControl
        w="fit-content"
        value={period}
        onChange={(value) => setPeriod(value as Period)}
        data={PERIODS.map((value) => ({ value, label: t(`household.periods.${value}`) }))}
      />
      {household.isPending ? (
        <Loader size="sm" />
      ) : household.isError ? (
        <Text c="red">{errorMessage(t, household.error)}</Text>
      ) : household.data.length === 0 ? (
        <Text c="dimmed">{t('household.empty')}</Text>
      ) : (
        <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }}>
          {household.data.map((totals, i) => (
            <PersonCard key={totals.person.id} rank={i + 1} totals={totals} you={totals.person.id === me?.user.id} />
          ))}
        </SimpleGrid>
      )}
    </Stack>
  );
}

function PersonCard({ rank, totals, you }: { rank: number; totals: PersonTotals; you: boolean }) {
  const { t } = useTranslation();
  const facts = [
    t('stats.sessions', { count: totals.sessions }),
    t('household.days', { count: totals.days }),
    t('household.games', { count: totals.games }),
    t('household.finished', { count: totals.finished }),
  ];

  return (
    <Card withBorder radius="md" data-testid={`person-${totals.person.id}`}>
      <Stack gap="xs">
        <Group justify="space-between" wrap="nowrap">
          <Group gap="xs" wrap="nowrap" className={classes.name}>
            <Text fw={700} c="dimmed" aria-label={t('household.rank', { rank })}>
              {rank}
            </Text>
            <Text fw={650} truncate>
              {totals.person.name ?? t('game.someone')}
            </Text>
            {you && (
              <Badge size="sm" variant="light">
                {t('household.you')}
              </Badge>
            )}
          </Group>
          <Text fw={700}>{playTime(totals.minutes, t)}</Text>
        </Group>
        <Text size="sm" c="dimmed">
          {facts.join(t('common.separator'))}
        </Text>
        {totals.top_game && (
          <Text size="sm">
            {t('household.topGame')}{' '}
            <Link to="/games/$gameId" params={{ gameId: String(totals.top_game.game_id) }} className={classes.link}>
              {totals.top_game.title}
            </Link>
          </Text>
        )}
      </Stack>
    </Card>
  );
}
