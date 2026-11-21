import { describe, expect, it } from 'vitest';
import i18n from '../../i18n';
import { describePlayers, effectiveCounts, type PlayerCounts } from './players';

const none: PlayerCounts = { local_max_players: null, online_max_players: null, local_coop: null, online_coop: null };
const say = (counts: Partial<PlayerCounts>) => describePlayers(i18n.t, { ...none, ...counts });

describe('describePlayers', () => {
  it('says unknown when nothing is known', () => {
    expect(say({})).toBe('Unknown locally, Unknown online');
  });

  it('reads a missing online count next to a local one as no online play', () => {
    expect(say({ local_max_players: 4 })).toBe('4 on the same screen, no online play');
    expect(say({ local_max_players: 2, online_coop: true })).toBe('2 on the same screen, Unknown online (co-op)');
  });

  it('reads a missing local count next to an online one as one player', () => {
    expect(say({ online_max_players: 8 })).toBe('1 locally, 8 online');
  });

  it('says on the same screen only for two or more local players', () => {
    expect(say({ local_max_players: 1, online_max_players: 2, online_coop: true })).toBe('1 locally, 2 online (co-op)');
  });

  it('marks co-op after each side', () => {
    expect(say({ local_max_players: 2, local_coop: true, online_max_players: 4 })).toBe(
      '2 on the same screen (co-op), 4 online',
    );
  });
});

describe('effectiveCounts', () => {
  it('takes the release value first, then the game value', () => {
    const game = { ...none, local_max_players: 4, online_max_players: 8 };
    expect(effectiveCounts({ ...none, local_max_players: 2 }, game)).toEqual({
      ...none,
      local_max_players: 2,
      online_max_players: 8,
    });
  });
});
