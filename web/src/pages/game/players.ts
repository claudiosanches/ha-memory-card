import type { TFunction } from 'i18next';
import type { GameDetail } from '../../api/types';

export type PlayerCounts = Pick<GameDetail, 'local_max_players' | 'online_max_players' | 'local_coop' | 'online_coop'>;

/** A release's values, each falling back to the game's. */
export function effectiveCounts(release: PlayerCounts, game: PlayerCounts): PlayerCounts {
  return {
    local_max_players: release.local_max_players ?? game.local_max_players,
    online_max_players: release.online_max_players ?? game.online_max_players,
    local_coop: release.local_coop ?? game.local_coop,
    online_coop: release.online_coop ?? game.online_coop,
  };
}

export function hasPlayerData(counts: PlayerCounts): boolean {
  return (
    counts.local_max_players != null ||
    counts.online_max_players != null ||
    counts.local_coop === true ||
    counts.online_coop === true
  );
}

/**
 * IGDB's multiplayer entries leave out what a game does not support, so a
 * count on one side with nothing on the other means that side has none.
 */
export function describePlayers(t: TFunction, counts: PlayerCounts): string {
  const local = counts.local_max_players ?? null;
  const online = counts.online_max_players ?? null;
  const noOnline = local != null && online == null && !counts.online_coop;
  const soloLocally = online != null && local == null && !counts.local_coop;

  const value = (max: number | null, coop?: boolean | null) => {
    const shown = max ?? t('game.playersUnknown');
    return coop ? t('game.withCoop', { value: shown }) : String(shown);
  };
  return t('game.playersSummary', {
    local: t('game.localPlayers', { value: value(soloLocally ? 1 : local, counts.local_coop) }),
    online: noOnline ? t('game.noOnline') : t('game.onlinePlayers', { value: value(online, counts.online_coop) }),
  });
}
