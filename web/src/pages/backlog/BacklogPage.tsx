import { Badge, Button, Group, Loader, Progress, Text, Title } from '@mantine/core';
import {
  IconCheck,
  IconPlayerPause,
  IconPlayerPlay,
  IconSparkles,
  IconTrophy,
  IconX,
} from '@tabler/icons-react';
import { Link } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { errorMessage } from '../../api/errors';
import { useGames, usePlatforms } from '../../api/queries';
import type { BuiltinStatus, GameSummary, StatusGroup } from '../../api/types';
import { PageHeader } from '../../components/PageHeader';
import { Description } from '../../components/Description';
import { Cover } from '../../components/Cover';
import { usePlayStatusList } from '../../components/playStatuses';
import { formatDate } from '../../components/status';
import classes from './BacklogPage.module.css';
import { Picker } from './Picker';
import { SortModal } from './SortModal';
import { useSetPlayStatus } from './usePlayStatus';

type Groups = Record<StatusGroup | 'unsorted', GameSummary[]>;

/** Groups games by what their status counts as. */
function groupByStatus(games: GameSummary[], groupOf: (key: string) => StatusGroup | undefined): Groups {
  const groups: Groups = { unsorted: [], backlog: [], playing: [], paused: [], finished: [], not_counted: [] };
  for (const game of games) {
    const group = game.play_status ? groupOf(game.play_status) : 'unsorted';
    if (group) groups[group].push(game);
  }
  return groups;
}

export function BacklogPage() {
  const { t } = useTranslation();
  // The library's default query, so both pages share one cached list.
  const games = useGames({});
  const platforms = usePlatforms();
  const statuses = usePlayStatusList();
  const [sorting, setSorting] = useState<GameSummary[] | null>(null);

  const groups = useMemo(
    () => groupByStatus(games.data ?? [], (key) => statuses.get(key)?.counts_as),
    [games.data, statuses],
  );
  const platformNames = useMemo(
    () => new Map((platforms.data ?? []).map((p) => [p.id, p.abbreviation ?? p.name])),
    [platforms.data],
  );

  if (games.isPending || statuses.isPending) return <Loader size="sm" />;
  if (games.isError) return <Text c="red">{errorMessage(t, games.error)}</Text>;

  const cleared = groups.finished.length;
  const open = groups.backlog.length + groups.playing.length + groups.paused.length;
  const total = cleared + open;

  return (
    <div className={classes.page}>
      <PageHeader title={t('backlog.title')} description={t('backlog.intro')}>
        {total > 0 && (
          <div className={classes.progress}>
            <Text size="sm" fw={600}>
              {t('backlog.progress', { cleared, total })}
            </Text>
            <Progress value={(cleared / total) * 100} size="lg" radius="xl" aria-hidden />
          </div>
        )}
      </PageHeader>

      {groups.unsorted.length > 0 && (
        <section className={classes.unsorted}>
          <IconSparkles size={28} stroke={1.6} aria-hidden />
          <div className={classes.unsortedText}>
            <Text fw={650}>{t('backlog.unsortedTitle')}</Text>
            <Description>
              {t('backlog.unsortedText', { count: groups.unsorted.length })}
            </Description>
          </div>
          <Button onClick={() => setSorting(groups.unsorted)}>{t('backlog.sort')}</Button>
        </section>
      )}

      {groups.backlog.length > 0 && <Picker />}

      {groups.playing.length > 0 && (
        <Section title={t('backlog.playing')} count={groups.playing.length}>
          <RowList games={groups.playing} platformNames={platformNames} actions={PLAYING_ACTIONS} />
        </Section>
      )}

      {groups.paused.length > 0 && (
        <Section title={t('backlog.paused')} count={groups.paused.length}>
          <RowList games={groups.paused} platformNames={platformNames} actions={PAUSED_ACTIONS} />
        </Section>
      )}

      <Section title={t('backlog.queue')} count={groups.backlog.length}>
        {groups.backlog.length === 0 ? (
          <div className={classes.empty}>
            {cleared > 0 && open === 0 && <IconTrophy size={32} stroke={1.5} aria-hidden />}
            <Text c="dimmed">{cleared > 0 && open === 0 ? t('backlog.cleared') : t('backlog.empty')}</Text>
          </div>
        ) : (
          <ul className={classes.grid}>
            {groups.backlog.map((game) => (
              <BacklogCard key={game.id} game={game} platformNames={platformNames} />
            ))}
          </ul>
        )}
      </Section>

      {sorting && <SortModal games={sorting} platformNames={platformNames} onClose={() => setSorting(null)} />}
    </div>
  );
}

function Section({ title, count, children }: { title: string; count: number; children: React.ReactNode }) {
  return (
    <section className={classes.section}>
      <Group gap="xs" align="baseline">
        <Title order={2} className={classes.sectionTitle}>
          {title}
        </Title>
        <Text c="dimmed" size="sm">
          {count}
        </Text>
      </Group>
      {children}
    </section>
  );
}

type Action = { status: BuiltinStatus; label: 'complete' | 'master' | 'pause' | 'resume' | 'drop'; icon: typeof IconCheck };

const PLAYING_ACTIONS: Action[] = [
  { status: 'completed', label: 'complete', icon: IconCheck },
  { status: 'mastered', label: 'master', icon: IconTrophy },
  { status: 'paused', label: 'pause', icon: IconPlayerPause },
  { status: 'dropped', label: 'drop', icon: IconX },
];

const PAUSED_ACTIONS: Action[] = [
  { status: 'playing', label: 'resume', icon: IconPlayerPlay },
  { status: 'dropped', label: 'drop', icon: IconX },
];

function platformList(game: GameSummary, names: Map<number, string>) {
  return game.platforms.map((p) => names.get(p.platform_id)).filter(Boolean);
}

const ROWS_PER_PAGE = 10;

/** Shows rows in pages, so a long list does not push the backlog off the screen. */
function RowList({
  games,
  platformNames,
  actions,
}: {
  games: GameSummary[];
  platformNames: Map<number, string>;
  actions: Action[];
}) {
  const { t } = useTranslation();
  const [limit, setLimit] = useState(ROWS_PER_PAGE);
  const hidden = games.length - limit;
  return (
    <>
      <ul className={classes.rows}>
        {games.slice(0, limit).map((game) => (
          <GameRow key={game.id} game={game} platformNames={platformNames} actions={actions} />
        ))}
      </ul>
      {hidden > 0 && (
        <Button variant="subtle" onClick={() => setLimit(limit + ROWS_PER_PAGE)} className={classes.more}>
          {t('backlog.showMore', { count: Math.min(hidden, ROWS_PER_PAGE), left: hidden })}
        </Button>
      )}
    </>
  );
}

/** The status name, for statuses the user added, since the section title only names the group. */
function CustomStatus({ game }: { game: GameSummary }) {
  const statuses = usePlayStatusList();
  const status = game.play_status ? statuses.get(game.play_status) : undefined;
  if (!status || status.builtin) return null;
  return (
    <Badge size="xs" variant="light" color={status.color} tt="none" className={classes.customStatus}>
      {status.name}
    </Badge>
  );
}

function GameRow({
  game,
  platformNames,
  actions,
}: {
  game: GameSummary;
  platformNames: Map<number, string>;
  actions: Action[];
}) {
  const { t, i18n } = useTranslation();
  const setStatus = useSetPlayStatus();
  return (
    <li className={classes.row} data-testid={`row-${game.id}`}>
      <Link to="/games/$gameId" params={{ gameId: String(game.id) }} className={classes.rowLink}>
        <Cover path={game.cover_path} title={game.title} width={56} />
        <div className={classes.rowText}>
          <Text fw={600} className={classes.gameTitle}>
            {game.title}
          </Text>
          <CustomStatus game={game} />
          <Text size="xs" c="dimmed">
            {platformList(game, platformNames).join(t('common.separator'))}
            {game.last_played_at &&
              t('common.separator') + t('backlog.lastPlayed', { date: formatDate(game.last_played_at, i18n.language) })}
          </Text>
        </div>
      </Link>
      <Group gap={6} className={classes.rowActions}>
        {actions.map(({ status, label, icon: Icon }) => (
          <Button
            key={status}
            size="xs"
            variant={status === 'dropped' ? 'subtle' : 'light'}
            color={status === 'dropped' ? 'gray' : undefined}
            leftSection={<Icon size={14} />}
            disabled={setStatus.isPending}
            onClick={() => setStatus.mutate({ id: game.id, status })}
          >
            {t(`backlog.${label}`)}
          </Button>
        ))}
      </Group>
    </li>
  );
}

function BacklogCard({ game, platformNames }: { game: GameSummary; platformNames: Map<number, string> }) {
  const { t } = useTranslation();
  const setStatus = useSetPlayStatus();
  return (
    <li className={classes.card} data-testid={`backlog-${game.id}`}>
      <Link to="/games/$gameId" params={{ gameId: String(game.id) }} className={classes.cardLink}>
        <div className={classes.artwork}>
          <Cover path={game.cover_path} title={game.title} />
        </div>
        <Text fw={600} size="sm" className={classes.gameTitle}>
          {game.title}
        </Text>
        <CustomStatus game={game} />
        <Text size="xs" c="dimmed">
          {platformList(game, platformNames).join(t('common.separator'))}
        </Text>
      </Link>
      <Button
        size="xs"
        variant="light"
        leftSection={<IconPlayerPlay size={14} />}
        disabled={setStatus.isPending}
        onClick={() => setStatus.mutate({ id: game.id, status: 'playing' })}
        aria-label={t('backlog.startNamed', { title: game.title })}
      >
        {t('backlog.startShort')}
      </Button>
    </li>
  );
}
