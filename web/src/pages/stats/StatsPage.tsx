import { Badge, Card, Loader, SegmentedControl, Stack, Text, Title, Tooltip } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import type { TFunction } from 'i18next';
import { type ReactNode, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, usePlatforms, useStores } from '../../api/queries';
import type { Stats } from '../../api/types';
import { formatMoney } from '../../components/money';
import { PageHeader } from '../../components/PageHeader';
import { usePlayStatusList } from '../../components/playStatuses';
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

export function StatsPage() {
  const { t, i18n } = useTranslation();
  const stats = useStats();

  return (
    <Stack gap={24}>
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
  const money = (amount: string, currency: string) => formatMoney(amount, currency, locale);
  const number = (value: number) => value.toLocaleString(locale);

  const platformName = new Map((platforms.data ?? []).map((p) => [p.id, p.name]));
  const storeName = new Map((stores.data ?? []).map((s) => [s.id, s.name]));
  const [main, ...others] = stats.spent;

  return (
    <>
      <dl className={classes.tiles}>
        <StatTile label={t('stats.games')} value={number(stats.games)} />
        <StatTile
          label={t('stats.copies')}
          value={number(stats.copies)}
          detail={t('stats.formats', { physical: number(stats.physical), digital: number(stats.digital) })}
        />
        <StatTile
          label={t('stats.value')}
          value={main ? money(main.amount, main.currency) : t('stats.noPrices')}
          detail={[
            ...others.map((other) => money(other.amount, other.currency)),
            stats.unpriced > 0 ? t('stats.unpriced', { count: stats.unpriced }) : null,
          ]
            .filter(Boolean)
            .join(t('common.separator'))}
          testId="stats-value"
        />
        <StatTile
          label={t('stats.wishlist')}
          value={number(stats.wishlist)}
          detail={stats.wishlist_value.map((v) => t('stats.expected', { amount: money(v.amount, v.currency) })).join(t('common.separator'))}
        />
        <StatTile
          label={t('stats.played')}
          value={playTime(stats.total_minutes, t)}
          detail={t('stats.sessions', { count: stats.total_sessions })}
        />
      </dl>

      <div className={classes.grid}>
        <Section title={t('stats.byPlatform')}>
          <MeterList
            rows={[...stats.platforms]
              .sort((a, b) => b.copies - a.copies)
              .map((row) => ({
                key: String(row.platform_id),
                label: platformName.get(row.platform_id) ?? '',
                value: row.copies,
                figure: t('stats.copiesCount', { count: row.copies }),
              }))}
          />
        </Section>

        <Section title={t('stats.byStatus')}>
          <MeterList
            rows={stats.play_statuses.map((row) => ({
              key: row.play_status ?? '',
              label: (
                <>
                  <Badge size="xs" circle color={row.play_status ? statuses.color(row.play_status) : 'gray'} />
                  {row.play_status ? statuses.name(row.play_status) : t('common.notSet')}
                </>
              ),
              value: row.games,
              figure: t('stats.gamesCount', { count: row.games }),
            }))}
          />
        </Section>

        {stats.spent_by_year.length > 0 && (
          <Section title={t('stats.spentByYear')}>
            <SpentByYear stats={stats} money={money} />
          </Section>
        )}

        <Section title={t('stats.byStore')}>
          <MeterList
            rows={stats.stores.map((row) => ({
              key: String(row.store_id),
              label: row.store_id ? (storeName.get(row.store_id) ?? '') : t('stats.noStore'),
              value: row.copies,
              figure: t('stats.copiesCount', { count: row.copies }),
            }))}
          />
        </Section>

        <Section title={t('stats.activity')}>
          {stats.total_sessions === 0 ? (
            <Text size="sm" c="dimmed">{t('stats.noSessions')}</Text>
          ) : (
            <Activity stats={stats} locale={locale} />
          )}
        </Section>

        {stats.most_played.length > 0 && (
          <Section title={t('stats.mostPlayed')}>
            <MeterList
              rows={stats.most_played.map((row) => ({
                key: String(row.game_id),
                label: (
                  <Link to="/games/$gameId" params={{ gameId: String(row.game_id) }} className={classes.gameLink}>
                    {row.title}
                  </Link>
                ),
                value: row.sessions,
                figure: [t('stats.sessions', { count: row.sessions }), row.minutes > 0 ? playTime(row.minutes, t) : null]
                  .filter(Boolean)
                  .join(t('common.separator')),
              }))}
            />
          </Section>
        )}
      </div>
    </>
  );
}

function StatTile({ label, value, detail, testId }: { label: string; value: string; detail?: string; testId?: string }) {
  return (
    <div className={classes.tile} data-testid={testId}>
      <dt>{label}</dt>
      <dd className={classes.tileValue}>{value}</dd>
      {detail && <dd className={classes.tileDetail}>{detail}</dd>}
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Card withBorder component="section" className={classes.section}>
      <Title order={2} size="h4" mb="sm">
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

function SpentByYear({ stats, money }: { stats: Stats; money: (amount: string, currency: string) => string }) {
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

/** Sessions per month for the last year, as columns. */
function Activity({ stats, locale }: { stats: Stats; locale: string }) {
  const { t } = useTranslation();
  const byMonth = new Map(stats.sessions_by_month.map((row) => [row.month, row]));
  const months = lastMonths().map((month) => {
    const [year, number] = month.split('-').map(Number);
    const date = new Date(year!, number! - 1, 1);
    return {
      month,
      short: date.toLocaleDateString(locale, { month: 'narrow' }),
      long: date.toLocaleDateString(locale, { month: 'long', year: 'numeric' }),
      sessions: byMonth.get(month)?.sessions ?? 0,
      minutes: byMonth.get(month)?.minutes ?? 0,
    };
  });
  const max = Math.max(1, ...months.map((m) => m.sessions));
  const peak = months.reduce((best, m) => (m.sessions > best.sessions ? m : best), months[0]!);

  return (
    <div className={classes.columns} role="list" aria-label={t('stats.activity')}>
      {months.map((m) => {
        const label = [m.long, t('stats.sessions', { count: m.sessions }), m.minutes > 0 ? playTime(m.minutes, t) : null]
          .filter(Boolean)
          .join(t('common.separator'));
        return (
          <Tooltip key={m.month} label={label} withArrow>
            <div className={classes.column} role="listitem" aria-label={label} tabIndex={0}>
              <span className={classes.columnValue}>{m === peak && m.sessions > 0 ? m.sessions : ''}</span>
              <span className={classes.plot}>
                <span className={classes.columnBar} style={{ height: `${(m.sessions / max) * 100}%` }} />
              </span>
              <span className={classes.columnLabel}>{m.short}</span>
            </div>
          </Tooltip>
        );
      })}
    </div>
  );
}
