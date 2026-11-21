import { Button, Group, Select, Text, Title } from '@mantine/core';
import { IconDice5, IconPlayerPlay } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { useDevices, usePlatforms } from '../../api/queries';
import type { Suggestion } from '../../api/types';
import { Description } from '../../components/Description';
import { Cover } from '../../components/Cover';
import classes from './BacklogPage.module.css';
import { useSetPlayStatus } from './usePlayStatus';

/** Rolls a random backlog game, optionally one a device can play. Managers only: rolls are shared history. */
export function Picker() {
  const { t } = useTranslation();
  const devices = useDevices();
  const [device, setDevice] = useState<string | null>(null);
  // Kept apart from the mutation, which drops its data while the next roll loads.
  const [picked, setPicked] = useState<{ suggestion: Suggestion; roll: number } | null>(null);
  const setStatus = useSetPlayStatus();

  const pick = useMutation({
    mutationFn: () => call(api.POST('/api/suggestions', { body: device ? { device: [Number(device)] } : {} })),
    onSuccess: (suggestion) => setPicked((last) => ({ suggestion, roll: (last?.roll ?? 0) + 1 })),
    onError: () => setPicked(null),
  });

  const start = (id: number) =>
    setStatus.mutate(
      { id, status: 'playing' },
      {
        onSuccess: () => {
          setPicked(null);
          pick.reset();
        },
      },
    );

  return (
    <section className={classes.picker}>
      <div className={classes.pickerIntro}>
        <Title order={2} className={classes.sectionTitle}>
          {t('backlog.pickTitle')}
        </Title>
        <Description>
          {t('backlog.pickText')}
        </Description>
        <Group className={classes.pickerControls} gap="sm" align="flex-end" mt="xs">
          <Select
            label={t('backlog.device')}
            placeholder={t('backlog.anyDevice')}
            data={(devices.data ?? [])
              .filter((d) => d.active)
              .map((d) => ({ value: String(d.id), label: d.name }))}
            value={device}
            onChange={setDevice}
            clearable
            className={classes.deviceSelect}
          />
          <Button
            leftSection={<IconDice5 size={18} className={pick.isPending ? classes.rolling : undefined} />}
            onClick={() => pick.mutate()}
            disabled={pick.isPending}
          >
            {picked ? t('backlog.pickAgain') : t('backlog.pick')}
          </Button>
        </Group>
      </div>

      <div
        className={classes.pickerResult}
        data-empty={(!picked && !pick.isError) || undefined}
        aria-live="polite"
        aria-busy={pick.isPending}
      >
        {picked ? (
          <Picked
            key={picked.roll}
            suggestion={picked.suggestion}
            rolling={pick.isPending}
            starting={setStatus.isPending}
            onStart={() => start(picked.suggestion.game.id)}
          />
        ) : pick.isError ? (
          <Text size="sm" c="dimmed">
            {errorMessage(t, pick.error)}
          </Text>
        ) : (
          <div className={classes.pickSlot} aria-hidden>
            <IconDice5 size={28} stroke={1.4} className={pick.isPending ? classes.rolling : undefined} />
          </div>
        )}
      </div>
    </section>
  );
}

type PickedProps = { suggestion: Suggestion; rolling: boolean; starting: boolean; onStart: () => void };

function Picked({ suggestion, rolling, starting, onStart }: PickedProps) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const devices = useDevices();
  const { game, copy_ids } = suggestion;

  const where = new Set<string>();
  for (const release of game.releases) {
    for (const copy of release.copies) {
      if (!copy_ids.includes(copy.id)) continue;
      const platform = platforms.data?.find((p) => p.id === release.platform_id)?.name;
      const device = devices.data?.find((d) => d.id === copy.device_id)?.name;
      where.add([platform, t(`format.${copy.format}`), device].filter(Boolean).join(t('common.separator')));
    }
  }

  return (
    <div className={classes.picked} data-rolling={rolling || undefined} data-testid="picked">
      <Link to="/games/$gameId" params={{ gameId: String(game.id) }} className={classes.pickedCover}>
        <Cover path={game.cover_path} title={game.title} width={96} />
      </Link>
      <div className={classes.pickedText}>
        <Text fw={700} size="lg" lineClamp={2} className={classes.gameTitle}>
          {game.title}
        </Text>
        {[...where].map((line) => (
          <Text key={line} size="sm" c="dimmed">
            {line}
          </Text>
        ))}
        <Group className={classes.pickedActions}>
          <Button leftSection={<IconPlayerPlay size={16} />} onClick={onStart} loading={starting} disabled={rolling}>
            {t('backlog.start')}
          </Button>
        </Group>
      </div>
    </div>
  );
}
