import { Badge, Group, Stack, Text } from '@mantine/core';
import { useTranslation } from 'react-i18next';
import type { GameDetail } from '../../api/types';
import { usePlayStatusList } from '../../components/playStatuses';
import { useFormatDate } from '../../hooks/useFormatDate';

/** How everyone else in the house plays the game. Only they change it. */
export function Household({ game }: { game: GameDetail }) {
  const { t } = useTranslation();
  const statuses = usePlayStatusList();
  const formatDate = useFormatDate();
  if (game.household.length === 0) return null;

  return (
    <Stack gap="xs" data-testid="household">
      <Text size="sm" fw={600}>
        {t('game.household')}
      </Text>
      {game.household.map((play) => (
        <Group key={play.person.id} justify="space-between" wrap="nowrap" gap="xs">
          <Text size="sm" truncate>
            {play.person.name ?? t('game.someone')}
          </Text>
          <Group gap={6} wrap="nowrap">
            {play.play_status && (
              <Badge size="sm" variant="light" color={statuses.color(play.play_status)}>
                {statuses.name(play.play_status)}
              </Badge>
            )}
            <Text size="xs" c="dimmed">
              {[
                t('stats.sessions', { count: play.sessions }),
                play.last_played_on ? formatDate(play.last_played_on) : null,
              ]
                .filter(Boolean)
                .join(t('common.separator'))}
            </Text>
          </Group>
        </Group>
      ))}
    </Stack>
  );
}
