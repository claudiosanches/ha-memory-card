import { Badge, Card, Loader, SegmentedControl, Stack, Text, Title, Tooltip } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { Link, useNavigate } from '@tanstack/react-router';
import type { TFunction } from 'i18next';
import { type ReactNode, useId, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, usePlatforms, useStores } from '../../api/queries';
import type { Stats, StatusGroup } from '../../api/types';
import { formatMoney } from '../../components/money';
import { PageHeader } from '../../components/PageHeader';
import { usePlayStatusList } from '../../components/playStatuses';
import { NO_STATUS } from '../library/search';
import charts from './charts.module.css';
import { Columns } from './Columns';
import { DonutChart } from './DonutChart';
import { Legend, shareFormat } from './Legend';
import { PlayCalendar } from './PlayCalendar';
import { SplitBars } from './SplitBars';
import classes from './StatsPage.module.css';

export function useStats() {
  return useQuery({ queryKey: queryKeys.stats, queryFn: () => call(api.GET('/api/stats')) });
}

/** Minutes as hours once there is an hour or more. */
export function playTime(minutes: number, t: TFunction): string {
  return minutes < 60
    ? t('sessions.minutes', { count: minutes })
    : t('stats.hours', { count: Math.round(minutes / 60) });
}

/** The last twelve months as YYYY-MM, oldest first. */
function lastMonths(now = new Date()): string[] {
  return Array.from({ length: 12 }, (_, i) => {
    const date = new Date(now.getFullYear(), now.getMonth() - 11 + i, 1);
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
  });
}

type Money = (amount: string, currency: string) => string;

export function StatsPage() {
  const { t, i18n } = useTranslation();
  const stats = useStats();

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.stats')} description={t('stats.intro')} />
      {stats.isPending ? (
        <Loader size="sm" />
      ) : stats.isError ? (
        <Text c="red">{errorMessage(t, stats.error)}</Text>
      ) : stats.data.copies === 0 && stats.data.wishlist === 0 ? (
        <Text c="dimmed">{t('stats.empty')}</Text>
      ) : (
        <StatsView stats={stats.data} locale={i18n.language} />
      )}
    </Stack>
  );
}

function StatsView({ stats, locale }: { stats: Stats; locale: string }) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const stores = useStores();
  const statuses = usePlayStatusList();
  const money: Money = (amount, currency) => formatMoney(amount, currency, locale);
  const number = (value: number) => value.toLocaleString(locale);
  const copies = (count: number) => t('stats.copiesCount', { count });

  const platformName = new Map((platforms.data ?? []).map((p) => [p.id, p.name]));
  const storeName = new Map((stores.data ?? []).map((s) => [s.id, s.name]));
  const amounts = (list: Stats['spent'], missing: string | null) =>
    [...list.slice(1).map((other) => money(other.amount, other.currency)), missing]
      .filter(Boolean)
      .join(t('common.separator'));
  const [spent] = stats.spent;
  const [estimated] = stats.estimated;

  return (
    <Stack gap="section">
      <dl className={classes.tiles}>
        <StatTile label={t('stats.games')} value={number(stats.games)} to="/" />
        <StatTile
          label={t('stats.copies')}
          value={number(stats.copies)}
          detail={t('stats.formats', { physical: number(stats.physical), digital: number(stats.digital) })}
        />
        <StatTile
          label={t('stats.wishlist')}
          value={number(stats.wishlist)}
          to="/wishlist"
          detail={stats.wishlist_value.map((v) => t('stats.expected', { amount: money(v.amount, v.currency) })).join(t('common.separator'))}
        />
      </dl>

      <Group title={t('stats.groupPlay')}>
        <div className={classes.grid}>
          <Panel title={t('stats.progress')}>
            <Progress stats={stats} locale={locale} />
          </Panel>

          <Panel title={t('stats.byStatus')}>
            <MeterList
              rows={stats.play_statuses.map((row) => ({
                key: row.play_status ?? '',
                label: (
                  <>
                    <Badge size="xs" circle color={row.play_status ? statuses.color(row.play_status) : 'gray'} />
                    <Link to="/" search={{ play_status: [row.play_status ?? NO_STATUS] }} className={charts.link}>
                      {row.play_status ? statuses.name(row.play_status) : t('common.notSet')}
                    </Link>
                  </>
                ),
                value: row.games,
                figure: t('stats.gamesCount', { count: row.games }),
              }))}
            />
          </Panel>

          {stats.most_played.length > 0 && (
            <Panel title={t('stats.mostPlayed')}>
              <MeterList
                rows={stats.most_played.map((row) => ({
                  key: String(row.game_id),
                  label: (
                    <Link to="/games/$gameId" params={{ gameId: String(row.game_id) }} className={charts.link}>
                      {row.title}
                    </Link>
                  ),
                  value: row.sessions,
                  figure: [t('stats.sessions', { count: row.sessions }), row.minutes > 0 ? playTime(row.minutes, t) : null]
                    .filter(Boolean)
                    .join(t('common.separator')),
                }))}
              />
            </Panel>
          )}
        </div>
        <PlayActivity stats={stats} locale={locale} />
      </Group>

      <Group title={t('stats.groupCollection')}>
        <div className={classes.grid}>
          <Panel title={t('stats.byPlatform')}>
            <DonutChart
              slices={stats.platforms.map((row) => ({
                key: String(row.platform_id),
                label: platformName.get(row.platform_id) ?? '',
                value: row.copies,
                search: { platform: [row.platform_id] },
              }))}
              figure={copies}
              locale={locale}
              empty={t('stats.noOwned')}
            />
          </Panel>

          {stats.physical > 0 && stats.digital > 0 && (
            <Panel title={t('stats.formatByPlatform')}>
              <SplitBars
                parts={[
                  { key: 'physical', label: t('format.physical'), color: 'var(--mc-chart-physical)' },
                  { key: 'digital', label: t('format.digital'), color: 'var(--mc-chart-digital)' },
                ]}
                rows={[...stats.platforms]
                  .sort((a, b) => b.copies - a.copies)
                  .map((row) => ({
                    key: String(row.platform_id),
                    label: platformName.get(row.platform_id) ?? '',
                    search: { platform: [row.platform_id] },
                    values: [row.physical, row.digital],
                    figure: t('stats.formats', { physical: number(row.physical), digital: number(row.digital) }),
                  }))}
              />
            </Panel>
          )}

          <Panel title={t('stats.byStore')}>
            <DonutChart
              slices={stats.stores.map((row) => ({
                key: String(row.store_id),
                label: row.store_id ? (storeName.get(row.store_id) ?? '') : t('stats.noStore'),
                value: row.copies,
                search: row.store_id ? { store: [row.store_id] } : undefined,
              }))}
              figure={copies}
              locale={locale}
              empty={t('stats.noOwned')}
            />
          </Panel>

          <Panel title={t('stats.byRegion')}>
            <DonutChart
              slices={stats.regions.map((row) => ({
                key: row.region,
                label: t(`region.${row.region}`),
                value: row.copies,
                search: { region: [row.region] },
              }))}
              figure={copies}
              locale={locale}
              empty={t('stats.noOwned')}
            />
          </Panel>

          {stats.physical > 0 && (
            <Panel title={t('stats.byCondition')}>
              <DonutChart
                slices={stats.conditions.map((row) => ({
                  key: row.condition ?? 'none',
                  label: row.condition ? t(`condition.${row.condition}`) : t('common.notSet'),
                  value: row.copies,
                }))}
                figure={copies}
                locale={locale}
                empty={t('stats.noOwned')}
              />
            </Panel>
          )}

          <Panel title={t('stats.added')}>
            <Added stats={stats} locale={locale} />
          </Panel>
        </div>
      </Group>

      <Group title={t('stats.groupMoney')}>
        <dl className={classes.tiles}>
          <StatTile
            label={t('stats.value')}
            value={spent ? money(spent.amount, spent.currency) : t('stats.noPrices')}
            empty={!spent}
            detail={amounts(stats.spent, stats.unpriced > 0 ? t('stats.unpriced', { count: stats.unpriced }) : null)}
            testId="stats-value"
          />
          <StatTile
            label={t('stats.estimated')}
            value={estimated ? money(estimated.amount, estimated.currency) : t('stats.noEstimates')}
            empty={!estimated}
            detail={amounts(stats.estimated, stats.unvalued > 0 ? t('stats.unvalued', { count: stats.unvalued }) : null)}
            testId="stats-estimated"
          />
        </dl>
        {(stats.spent_by_year.length > 0 || stats.most_valuable.length > 0) && (
          <div className={classes.grid}>
            {stats.spent_by_year.length > 0 && (
              <Panel title={t('stats.spentByYear')}>
                <SpentByYear stats={stats} money={money} />
              </Panel>
            )}

            {stats.most_valuable.length > 0 && (
              <Panel title={t('stats.valueTitle')}>
                <Value stats={stats} money={money} locale={locale} platformName={platformName} />
              </Panel>
            )}
          </div>
        )}
      </Group>
    </Stack>
  );
}

function PlayActivity({ stats, locale }: { stats: Stats; locale: string }) {
  const { t } = useTranslation();
  const id = useId();
  const today = new Date();
  const month = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}`;
  const monthDays = stats.play_days.filter(({ day }) => day.startsWith(month));
  const minutes = monthDays.reduce((sum, day) => sum + day.minutes, 0);
  const sessions = monthDays.reduce((sum, day) => sum + day.sessions, 0);

  return (
    <section aria-labelledby={id} className={classes.group}>
      <Title order={3} size="h4" id={id}>{t('stats.activity')}</Title>
      <dl className={classes.tiles}>
        <StatTile
          label={t('stats.lifetimePlay')}
          value={playTime(stats.total_minutes, t)}
          detail={stats.total_sessions > 0 ? t('stats.thisMonth', { value: playTime(minutes, t) }) : undefined}
        />
        <StatTile
          label={t('stats.lifetimeSessions')}
          value={stats.total_sessions.toLocaleString(locale)}
          detail={stats.total_sessions > 0 ? t('stats.thisMonth', { value: t('stats.sessions', { count: sessions }) }) : undefined}
        />
      </dl>
      {stats.total_sessions > 0 ? (
        <Card withBorder className={classes.panel}>
          <PlayCalendar stats={stats} locale={locale} playTime={playTime} />
        </Card>
      ) : (
        <Text size="sm" c="dimmed">{t('stats.noSessions')}</Text>
      )}
    </section>
  );
}

function StatTile({
  label,
  value,
  detail,
  empty,
  to,
  testId,
}: {
  label: string;
  value: string;
  detail?: string;
  /** The value is a sentence saying there is nothing yet. */
  empty?: boolean;
  to?: '/' | '/wishlist';
  testId?: string;
}) {
  return (
    <div className={classes.tile} data-testid={testId} data-link={to ? '' : undefined}>
      <dt>
        {to ? (
          <Link to={to} className={classes.tileLink}>
            {label}
          </Link>
        ) : (
          label
        )}
      </dt>
      <dd className={classes.tileValue} data-empty={empty ? '' : undefined}>{value}</dd>
      {detail && <dd className={classes.tileDetail}>{detail}</dd>}
    </div>
  );
}

/** A heading over related panels. */
function Group({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <section aria-labelledby={id} className={classes.group}>
      <Title order={2} size="h3" id={id}>
        {title}
      </Title>
      {children}
    </section>
  );
}

function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Card withBorder component="section" className={classes.panel}>
      <Title order={3} size="h4" mb="sm">
        {title}
      </Title>
      {children}
    </Card>
  );
}

type Row = { key: string; label: ReactNode; value: number; figure: string };

/** A name, a bar against the largest value, and the figure, per row. */
function MeterList({ rows }: { rows: Row[] }) {
  const max = Math.max(1, ...rows.map((row) => row.value));
  return (
    <ul className={classes.meters}>
      {rows.map((row) => (
        <li key={row.key} className={classes.meter}>
          <span className={classes.meterLabel}>{row.label}</span>
          <span className={classes.track} aria-hidden>
            <span className={classes.fill} style={{ width: `${(row.value / max) * 100}%` }} />
          </span>
          <span className={classes.figure}>{row.figure}</span>
        </li>
      ))}
    </ul>
  );
}

type Group = 'finished' | 'inProgress' | 'notStarted' | 'notCounted' | 'notSet';
// Not counted goes last, after not set, so with the default colors pink
// rarely sits beside green, which some readers cannot tell apart.
const GROUPS: { key: Group; color: string }[] = [
  { key: 'finished', color: 'var(--mc-chart-finished)' },
  { key: 'inProgress', color: 'var(--mc-chart-in-progress)' },
  { key: 'notStarted', color: 'var(--mc-chart-not-started)' },
  { key: 'notSet', color: 'var(--mc-chart-not-set)' },
  { key: 'notCounted', color: 'var(--mc-chart-not-counted)' },
];
const GROUP_OF: Record<StatusGroup, Group> = {
  finished: 'finished',
  playing: 'inProgress',
  paused: 'inProgress',
  backlog: 'notStarted',
  not_counted: 'notCounted',
};

/** How much of what is owned has been finished, by what each play status counts as. */
function Progress({ stats, locale }: { stats: Stats; locale: string }) {
  const { t } = useTranslation();
  const statuses = usePlayStatusList();
  const navigate = useNavigate();
  const [active, setActive] = useState<string | null>(null);

  const totals = new Map<Group, { games: number; keys: string[] }>();
  for (const row of stats.play_statuses) {
    const status = row.play_status ? statuses.get(row.play_status) : undefined;
    const group = status ? GROUP_OF[status.counts_as] : 'notSet';
    const entry = totals.get(group) ?? { games: 0, keys: [] };
    entry.games += row.games;
    entry.keys.push(row.play_status ?? NO_STATUS);
    totals.set(group, entry);
  }
  const parts = GROUPS.flatMap((group) => {
    const entry = totals.get(group.key);
    return entry && entry.games > 0 ? [{ ...group, ...entry }] : [];
  });
  const total = parts.reduce((sum, p) => sum + p.games, 0);
  if (total === 0) return <Text size="sm" c="dimmed">{t('stats.noOwned')}</Text>;

  const counted = total - (totals.get('notCounted')?.games ?? 0);
  const finished = totals.get('finished')?.games ?? 0;
  const share = shareFormat(locale, total);
  const percent = new Intl.NumberFormat(locale, { style: 'percent', maximumFractionDigits: 0 });

  return (
    <Stack gap="sm">
      <div>
        <Text className={classes.headline}>{percent.format(counted ? finished / counted : 0)}</Text>
        <Text size="sm" c="dimmed">
          {t('stats.finishedOf', { finished: finished.toLocaleString(locale), count: counted })}
        </Text>
      </div>
      <div className={classes.segments} aria-hidden>
        {parts.map((part) => (
          <Tooltip key={part.key} label={`${t(`stats.groups.${part.key}`)}: ${t('stats.gamesCount', { count: part.games })}`} withArrow>
            <span
              className={classes.segment}
              style={{ flexGrow: part.games, background: part.color }}
              data-dim={active && active !== part.key ? '' : undefined}
              onMouseEnter={() => setActive(part.key)}
              onMouseLeave={() => setActive(null)}
              onClick={() => void navigate({ to: '/', search: { play_status: part.keys } })}
            />
          </Tooltip>
        ))}
      </div>
      <Legend
        rows={parts.map((part) => ({
          key: part.key,
          label: t(`stats.groups.${part.key}`),
          color: part.color,
          figure: `${t('stats.gamesCount', { count: part.games })}${t('common.separator')}${share(part.games)}`,
          search: { play_status: part.keys },
        }))}
        active={active}
        onActive={setActive}
      />
    </Stack>
  );
}

/** Copies added per year since the first, or per month for the last year. */
function Added({ stats, locale }: { stats: Stats; locale: string }) {
  const { t } = useTranslation();
  const [view, setView] = useState<'months' | 'years'>('years');
  const copies = (count: number) => t('stats.copiesCount', { count });

  const byMonth = new Map(stats.added_by_month.map((row) => [row.period, row.copies]));
  const months = lastMonths().map((period) => {
    const [year, month] = period.split('-').map(Number);
    const date = new Date(year!, month! - 1, 1);
    const value = byMonth.get(period) ?? 0;
    return {
      key: period,
      short: date.toLocaleDateString(locale, { month: 'narrow' }),
      long: date.toLocaleDateString(locale, { month: 'long', year: 'numeric' }),
      value,
      figure: copies(value),
    };
  });
  const byYear = new Map(stats.added_by_year.map((row) => [Number(row.period), row.copies]));
  const first = Math.min(...byYear.keys());
  const last = Math.max(...byYear.keys());
  const years =
    byYear.size > 0
      ? Array.from({ length: last - first + 1 }, (_, i) => {
          const value = byYear.get(first + i) ?? 0;
          return { key: String(first + i), short: String(first + i), long: String(first + i), value, figure: copies(value) };
        })
      : [];
  const shown = view === 'years' && years.length > 1 ? 'years' : 'months';

  return (
    <Stack gap="sm">
      <Text size="sm" c="dimmed">{t('stats.addedHint')}</Text>
      {years.length > 1 && (
        <SegmentedControl
          size="xs"
          w="fit-content"
          value={shown}
          onChange={(value) => setView(value as 'months' | 'years')}
          data={[
            { value: 'years', label: t('stats.byYear') },
            { value: 'months', label: t('stats.lastYear') },
          ]}
        />
      )}
      <Columns columns={shown === 'years' ? years : months} label={t('stats.added')} />
    </Stack>
  );
}

function SpentByYear({ stats, money }: { stats: Stats; money: Money }) {
  const { t } = useTranslation();
  const currencies = [...new Set(stats.spent_by_year.map((row) => row.currency))];
  const [currency, setCurrency] = useState(stats.spent[0]?.currency ?? currencies[0]!);
  const shown = currencies.includes(currency) ? currency : currencies[0]!;
  const rows = stats.spent_by_year.filter((row) => row.currency === shown);
  return (
    <Stack gap="sm">
      {currencies.length > 1 && (
        <SegmentedControl size="xs" w="fit-content" data={currencies} value={shown} onChange={setCurrency} />
      )}
      <MeterList
        rows={rows.map((row) => ({
          key: row.year,
          label: row.year,
          value: Number(row.amount),
          figure: `${money(row.amount, row.currency)}${t('common.separator')}${t('stats.copiesCount', { count: row.copies })}`,
        }))}
      />
    </Stack>
  );
}

/** What copies cost against what they are worth, and the copies worth the most. */
function Value({
  stats,
  money,
  locale,
  platformName,
}: {
  stats: Stats;
  money: Money;
  locale: string;
  platformName: Map<number, string>;
}) {
  const { t } = useTranslation();
  const currencies = [...new Set([...stats.estimated, ...stats.value_change].map((row) => row.currency))];
  const [currency, setCurrency] = useState(currencies[0]!);
  const shown = currencies.includes(currency) ? currency : currencies[0]!;
  const change = stats.value_change.find((row) => row.currency === shown);
  const valuable = stats.most_valuable.filter((row) => row.currency === shown);
  const signed = (amount: string) => `${Number(amount) > 0 ? '+' : ''}${money(amount, shown)}`;
  const percent = new Intl.NumberFormat(locale, { style: 'percent', maximumFractionDigits: 0, signDisplay: 'exceptZero' });

  return (
    <Stack gap="md">
      {currencies.length > 1 && (
        <SegmentedControl size="xs" w="fit-content" data={currencies} value={shown} onChange={setCurrency} />
      )}
      {change && (
        <div>
          <dl className={classes.figures} data-testid="value-change">
            <div>
              <dt>{t('stats.paid')}</dt>
              <dd>{money(change.paid, shown)}</dd>
            </div>
            <div>
              <dt>{t('stats.worth')}</dt>
              <dd>{money(change.estimated, shown)}</dd>
            </div>
            <div>
              <dt>{t('stats.change')}</dt>
              <dd>
                {signed(change.change)}
                {Number(change.paid) > 0 && (
                  <span className={classes.figureNote}> {percent.format(Number(change.change) / Number(change.paid))}</span>
                )}
              </dd>
            </div>
          </dl>
          <Text size="sm" c="dimmed">{t('stats.valueCopies', { count: change.copies })}</Text>
        </div>
      )}
      {valuable.length > 0 && (
        <Stack gap="xs">
          <Title order={4} size="h5">{t('stats.mostValuable')}</Title>
          <MeterList
            rows={valuable.map((row) => ({
              key: String(row.copy_id),
              label: (
                <span className={classes.copyLabel}>
                  <Link to="/games/$gameId" params={{ gameId: String(row.game_id) }} className={charts.link}>
                    {row.title}
                  </Link>
                  <Text span size="xs" c="dimmed">{platformName.get(row.platform_id)}</Text>
                </span>
              ),
              value: Number(row.estimated),
              figure: [money(row.estimated, shown), row.paid ? t('stats.cost', { amount: money(row.paid, shown) }) : null]
                .filter(Boolean)
                .join(t('common.separator')),
            }))}
          />
        </Stack>
      )}
    </Stack>
  );
}
