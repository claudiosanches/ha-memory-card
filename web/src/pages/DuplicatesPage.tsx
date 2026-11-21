import { Badge, Group, Loader, Stack, Text, Title } from '@mantine/core';
import { Link } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { errorMessage } from '../api/errors';
import { useDevices, useDuplicates, usePlatforms, useStores } from '../api/queries';
import { DUPLICATE_LABELS, type DuplicateCopy, type DuplicateGroup, type DuplicateLabel } from '../api/types';
import { Cover } from '../components/Cover';
import classes from './DuplicatesPage.module.css';

export function DuplicatesPage() {
  const { t } = useTranslation();
  const duplicates = useDuplicates();
  const [label, setLabel] = useState<DuplicateLabel | null>(null);

  const all = duplicates.data ?? [];
  const shown = label ? all.filter((group) => group.labels.includes(label)) : all;
  const counts = new Map(DUPLICATE_LABELS.map((l) => [l, all.filter((g) => g.labels.includes(l)).length]));

  return (
    <Stack gap="lg">
      <Group justify="space-between" align="center" className={classes.heading}>
        <div>
          <Title order={1} className={classes.title}>
            {t('nav.duplicates')}
          </Title>
          <Text size="sm" c="dimmed">
            {t('duplicates.intro')}
          </Text>
        </div>
        {duplicates.data && (
          <Text className={classes.count} data-testid="duplicates-count" aria-live="polite">
            {t('duplicates.count', { count: shown.length })}
          </Text>
        )}
      </Group>

      {all.length > 0 && (
        <div className={classes.filters} role="group" aria-label={t('duplicates.filter')}>
          <button type="button" className={classes.filter} aria-pressed={!label} onClick={() => setLabel(null)}>
            {t('duplicates.all')}
            <span className={classes.filterCount}>{all.length}</span>
          </button>
          {DUPLICATE_LABELS.filter((l) => counts.get(l)).map((l) => (
            <button
              type="button"
              key={l}
              className={classes.filter}
              aria-pressed={label === l}
              onClick={() => setLabel(l)}
            >
              {t(`duplicates.label.${l}`)}
              <span className={classes.filterCount}>{counts.get(l)}</span>
            </button>
          ))}
        </div>
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
            <GroupCard key={group.game_id} group={group} />
          ))}
        </div>
      )}
    </Stack>
  );
}

function GroupCard({ group }: { group: DuplicateGroup }) {
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
        <ul className={classes.copies}>
          {group.copies.map((copy) => (
            <CopyLine key={copy.id} copy={copy} />
          ))}
        </ul>
      </div>
    </article>
  );
}

function CopyLine({ copy }: { copy: DuplicateCopy }) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const names = useMemo(
    () => ({
      platform: new Map((platforms.data ?? []).map((p) => [p.id, p.name])),
      device: new Map((devices.data ?? []).map((d) => [d.id, d.name])),
      store: new Map((stores.data ?? []).map((s) => [s.id, s.name])),
    }),
    [platforms.data, devices.data, stores.data],
  );
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
