import { Badge, Group, Loader, Stack, Text } from '@mantine/core';
import { Link } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { errorMessage } from '../api/errors';
import { useDevices, useDuplicates, usePlatforms, useStores } from '../api/queries';
import { DUPLICATE_LABELS, type DuplicateCopy, type DuplicateGroup, type DuplicateLabel } from '../api/types';
import { FilterBar, FilterButton } from '../components/FilterBar';
import { Cover } from '../components/Cover';
import classes from './DuplicatesPage.module.css';

type Names = { platform: Map<number, string>; device: Map<number, string>; store: Map<number, string> };

export function DuplicatesTab({ showIntro = true }: { showIntro?: boolean }) {
  const { t } = useTranslation();
  const duplicates = useDuplicates();
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const [label, setLabel] = useState<DuplicateLabel | null>(null);
  const names = useMemo<Names>(
    () => ({
      platform: new Map((platforms.data ?? []).map((p) => [p.id, p.name])),
      device: new Map((devices.data ?? []).map((d) => [d.id, d.name])),
      store: new Map((stores.data ?? []).map((s) => [s.id, s.name])),
    }),
    [platforms.data, devices.data, stores.data],
  );

  const all = duplicates.data ?? [];
  const shown = label ? all.filter((group) => group.labels.includes(label)) : all;
  const counts = new Map(DUPLICATE_LABELS.map((l) => [l, all.filter((g) => g.labels.includes(l)).length]));

  return (
    <Stack gap="card">
      {(showIntro || duplicates.data) && (
        <Group justify="space-between" align="baseline" gap="sm">
          {showIntro && (
            <Text size="sm" c="dimmed">
              {t('duplicates.intro')}
            </Text>
          )}
          {duplicates.data && (
            <Text size="sm" c="dimmed" data-testid="duplicates-count">
              {t('duplicates.count', { count: shown.length })}
            </Text>
          )}
        </Group>
      )}

      {all.length > 0 && (
        <FilterBar label={t('duplicates.filter')}>
          <FilterButton count={all.length} selected={!label} onClick={() => setLabel(null)}>
            {t('duplicates.all')}
          </FilterButton>
          {DUPLICATE_LABELS.filter((l) => counts.get(l)).map((l) => (
            <FilterButton
              key={l}
              count={counts.get(l)}
              selected={label === l}
              onClick={() => setLabel(l)}
            >
              {t(`duplicates.label.${l}`)}
            </FilterButton>
          ))}
        </FilterBar>
      )}

      {duplicates.isPending ? (
        <div className={classes.empty}>
          <Loader size="sm" />
        </div>
      ) : duplicates.isError ? (
        <Text c="red">{errorMessage(t, duplicates.error)}</Text>
      ) : all.length === 0 ? (
        <div className={classes.empty}>{t('duplicates.empty')}</div>
      ) : (
        <div className={classes.groups}>
          {shown.map((group) => (
            <GroupCard key={group.game_id} group={group} names={names} />
          ))}
        </div>
      )}
    </Stack>
  );
}

function GroupCard({ group, names }: { group: DuplicateGroup; names: Names }) {
  const { t } = useTranslation();
  const gameLink = { to: '/games/$gameId', params: { gameId: String(group.game_id) } } as const;
  return (
    <article className={classes.group} data-testid={`duplicate-${group.title}`}>
      <Link {...gameLink} className={classes.cover} tabIndex={-1} aria-hidden>
        <Cover path={group.cover_path} title={group.title} />
      </Link>
      <div className={classes.body}>
        <Link {...gameLink} className={classes.gameTitle}>
          {group.title}
        </Link>
        <Group gap={6}>
          {DUPLICATE_LABELS.filter((l) => group.labels.includes(l)).map((l) => (
            <Badge key={l} size="sm" variant="light" tt="none">
              {t(`duplicates.label.${l}`)}
            </Badge>
          ))}
        </Group>
      </div>
      <ul className={classes.copies}>
        {group.copies.map((copy) => (
          <CopyLine key={copy.id} copy={copy} names={names} />
        ))}
      </ul>
    </article>
  );
}

function CopyLine({ copy, names }: { copy: DuplicateCopy; names: Names }) {
  const { t } = useTranslation();
  const details = [
    t(`format.${copy.format}`),
    copy.device_id ? names.device.get(copy.device_id) : null,
    copy.store_id ? names.store.get(copy.store_id) : null,
    copy.condition ? t(`condition.${copy.condition}`) : null,
    copy.release_title,
  ].filter(Boolean);
  return (
    <li className={classes.copy}>
      <Text size="sm" fw={550}>
        {t('game.releaseHeading', {
          platform: names.platform.get(copy.platform_id) ?? '',
          region: t(`region.${copy.region}`),
        })}
      </Text>
      <Text size="xs" c="dimmed">
        {details.join(t('common.separator'))}
      </Text>
    </li>
  );
}
