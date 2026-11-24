import { Badge, Card, Loader, SegmentedControl, SimpleGrid, Stack, Text } from '@mantine/core';
import { useLocalStorage } from '@mantine/hooks';
import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useMe } from '../../api/queries';
import type { components } from '../../api/schema';
import { Cover } from '../../components/Cover';
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
  const { t, i18n } = useTranslation();
  const [stored, setPeriod] = useLocalStorage<Period>({ key: 'memory-card-household-period', defaultValue: 'month' });
  const period = PERIODS.includes(stored) ? stored : 'month';
  const household = useHousehold(period);
  const me = useMe().data;
  const locale = i18n.language;

  const people = household.data ?? [];
  const totalMinutes = people.reduce((sum, p) => sum + p.minutes, 0);
  const totalSessions = people.reduce((sum, p) => sum + p.sessions, 0);
  const totalFinished = people.reduce((sum, p) => sum + p.finished, 0);
  const activePlayers = people.filter((p) => p.sessions > 0 || p.minutes > 0).length;
  const maxMinutes = Math.max(0, ...people.map((p) => p.minutes));
  const maxSessions = Math.max(0, ...people.map((p) => p.sessions));

  const playersDetail =
    activePlayers < people.length
      ? t('household.activeOf', {
          active: activePlayers.toLocaleString(locale),
          total: people.length.toLocaleString(locale),
        })
      : t('household.people', { count: people.length });

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.household')} description={t('household.intro')}>
        <SegmentedControl
          className={classes.period}
          value={period}
          onChange={(value) => setPeriod(value as Period)}
          data={PERIODS.map((value) => ({ value, label: t(`household.periods.${value}`) }))}
        />
      </PageHeader>
      {household.isPending ? (
        <Loader size="sm" />
      ) : household.isError ? (
        <Text c="red">{errorMessage(t, household.error)}</Text>
      ) : people.length === 0 ? (
        <Text c="dimmed">{t('household.empty')}</Text>
      ) : (
        <>
          <dl className={classes.tiles} data-testid="household-summary">
            <div className={classes.tile}>
              <dt>{t('household.totalPlay')}</dt>
              <dd className={classes.tileValue}>{playTime(totalMinutes, t)}</dd>
              <dd className={classes.tileDetail}>{playersDetail}</dd>
            </div>
            <div className={classes.tile}>
              <dt>{t('household.sessions')}</dt>
              <dd className={classes.tileValue}>{totalSessions.toLocaleString(locale)}</dd>
            </div>
            <div className={classes.tile}>
              <dt>{t('household.gamesFinished')}</dt>
              <dd className={classes.tileValue}>{totalFinished.toLocaleString(locale)}</dd>
            </div>
          </dl>

          <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="card">
            {people.map((totals, i) => {
              const ratio =
                maxMinutes > 0
                  ? totals.minutes / maxMinutes
                  : maxSessions > 0
                    ? totals.sessions / maxSessions
                    : 0;
              return (
                <PersonCard
                  key={totals.person.id}
                  rank={i + 1}
                  totals={totals}
                  you={totals.person.id === me?.user.id}
                  ratio={ratio}
                  showBar={maxMinutes > 0 || maxSessions > 0}
                  locale={locale}
                />
              );
            })}
          </SimpleGrid>
        </>
      )}
    </Stack>
  );
}

function PersonCard({
  rank,
  totals,
  you,
  ratio,
  showBar,
  locale,
}: {
  rank: number;
  totals: PersonTotals;
  you: boolean;
  ratio: number;
  showBar: boolean;
  locale: string;
}) {
  const { t } = useTranslation();
  const hasPlay = totals.sessions > 0 || totals.minutes > 0;
  const quiet = !hasPlay && totals.finished === 0;
  const isLeader = rank === 1 && hasPlay;

  return (
    <Card
      withBorder
      radius="md"
      className={classes.card}
      data-testid={`person-${totals.person.id}`}
      data-you={you ? '' : undefined}
      data-quiet={quiet ? '' : undefined}
    >
      <div className={classes.header}>
        <div className={classes.identity}>
          <span
            className={classes.rank}
            data-leader={isLeader ? '' : undefined}
            aria-label={t('household.rank', { rank })}
          >
            {rank}
          </span>
          <Text fw={650} size="md" truncate className={classes.personName}>
            {totals.person.name ?? t('game.someone')}
          </Text>
          {you && (
            <Badge size="xs" variant="light" className={classes.you}>
              {t('household.you')}
            </Badge>
          )}
        </div>
        <Text className={classes.playTime} data-empty={!hasPlay ? '' : undefined}>
          {playTime(totals.minutes, t)}
        </Text>
      </div>

      {showBar && (
        <div className={classes.track} aria-hidden="true">
          <span
            className={classes.fill}
            style={{ width: `${ratio * 100}%` }}
            data-zero={ratio === 0 ? '' : undefined}
          />
        </div>
      )}

      {quiet ? (
        <Text size="sm" c="dimmed" className={classes.quietNote}>
          {t('household.noPlay')}
        </Text>
      ) : (
        <dl className={classes.stats}>
          <div>
            <dt>{t('household.sessions')}</dt>
            <dd>{totals.sessions.toLocaleString(locale)}</dd>
          </div>
          <div>
            <dt>{t('household.days')}</dt>
            <dd>{totals.days.toLocaleString(locale)}</dd>
          </div>
          <div>
            <dt>{t('household.games')}</dt>
            <dd>{totals.games.toLocaleString(locale)}</dd>
          </div>
          <div>
            <dt>{t('household.finished')}</dt>
            <dd>{totals.finished.toLocaleString(locale)}</dd>
          </div>
        </dl>
      )}

      {totals.top_game && (
        <div className={classes.footer}>
          <div className={classes.topGame}>
            <Cover path={totals.top_game.cover_path} title={totals.top_game.title} width={36} />
            <div className={classes.topGameMeta}>
              <span className={classes.topGameEyebrow}>{t('household.topGame')}</span>
              <Link
                to="/games/$gameId"
                params={{ gameId: String(totals.top_game.game_id) }}
                className={classes.topGameLink}
              >
                {totals.top_game.title}
              </Link>
              <span className={classes.topGameDetail}>
                {t('stats.sessions', { count: totals.top_game.sessions })}
              </span>
            </div>
          </div>
        </div>
      )}
    </Card>
  );
}
