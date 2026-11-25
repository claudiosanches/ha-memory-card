import { Loader, Modal } from '@mantine/core';
import { IconChevronLeft, IconChevronRight, IconDeviceSdCard, IconX } from '@tabler/icons-react';
import { useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useGames, usePlatforms } from '../api/queries';
import type { GameSummary } from '../api/types';
import { Cover } from './Cover';
import classes from './SlotOneModal.module.css';

const BLOCKS_PER_CARD = 15;

type Props = {
  opened: boolean;
  onClose: () => void;
};

/** Nostalgic 15-block PS1-style Memory Card browser hidden behind the header logo. */
export function SlotOneModal({ opened, onClose }: Props) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const games = useGames({}, opened);
  const platforms = usePlatforms();
  const [slot, setSlot] = useState(0);
  const [focused, setFocused] = useState<GameSummary | null>(null);

  const platformNames = useMemo(
    () => new Map((platforms.data ?? []).map((p) => [p.id, p.abbreviation ?? p.name])),
    [platforms.data],
  );

  const all = games.data ?? [];
  const totalSlots = Math.max(1, Math.ceil(all.length / BLOCKS_PER_CARD));
  const safeSlot = Math.min(slot, totalSlots - 1);
  const slice = all.slice(safeSlot * BLOCKS_PER_CARD, (safeSlot + 1) * BLOCKS_PER_CARD);
  const blocks: (GameSummary | null)[] = Array.from(
    { length: BLOCKS_PER_CARD },
    (_, i) => slice[i] ?? null,
  );

  const openGame = (game: GameSummary) => {
    onClose();
    void navigate({ to: '/games/$gameId', params: { gameId: String(game.id) } });
  };

  const platformsOf = (game: GameSummary) =>
    game.platforms.map((p) => platformNames.get(p.platform_id)).filter(Boolean).join(t('common.separator'));

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      withCloseButton={false}
      centered
      size="lg"
      aria-label={t('easterEggs.slotOne.title', { slot: safeSlot + 1 })}
      classNames={{ content: classes.modalContent, body: classes.modalBody }}
    >
      <div className={classes.shell} data-testid="slot-one-modal">
        <div className={classes.header}>
          <div className={classes.headerLeft}>
            <span className={classes.cardBadge} aria-hidden>
              <IconDeviceSdCard size={20} stroke={1.8} />
            </span>
            <div>
              <h2 className={classes.title}>{t('easterEggs.slotOne.title', { slot: safeSlot + 1 })}</h2>
              <div className={classes.subtitle}>
                {t('easterEggs.slotOne.blocks', { used: slice.length, total: BLOCKS_PER_CARD })}
              </div>
            </div>
          </div>

          <div className={classes.headerRight}>
            {totalSlots > 1 && (
              <div className={classes.slotControls}>
                <button
                  type="button"
                  className={classes.iconBtn}
                  aria-label={t('easterEggs.slotOne.prevSlot')}
                  disabled={safeSlot === 0}
                  onClick={() => { setSlot((s) => Math.max(0, s - 1)); setFocused(null); }}
                >
                  <IconChevronLeft size={16} />
                </button>
                <button
                  type="button"
                  className={classes.iconBtn}
                  aria-label={t('easterEggs.slotOne.nextSlot')}
                  disabled={safeSlot >= totalSlots - 1}
                  onClick={() => { setSlot((s) => Math.min(totalSlots - 1, s + 1)); setFocused(null); }}
                >
                  <IconChevronRight size={16} />
                </button>
              </div>
            )}
            <button
              type="button"
              className={classes.iconBtn}
              aria-label={t('easterEggs.slotOne.close')}
              onClick={onClose}
            >
              <IconX size={16} />
            </button>
          </div>
        </div>

        {games.isPending ? (
          <div style={{ display: 'grid', placeItems: 'center', minHeight: 240 }}>
            <Loader size="sm" color="#ffd859" />
          </div>
        ) : (
          <div className={classes.grid} role="list">
            {blocks.map((game, index) => {
              const num = String(index + 1).padStart(2, '0');
              if (!game) {
                return (
                  <div
                    key={`free-${index}`}
                    role="listitem"
                    className={`${classes.block} ${classes.free}`}
                    onMouseEnter={() => setFocused(null)}
                  >
                    <span className={classes.blockNum}>{num}</span>
                    <span className={classes.freeLabel}>{t('easterEggs.slotOne.free')}</span>
                  </div>
                );
              }
              return (
                <button
                  key={game.id}
                  type="button"
                  role="listitem"
                  className={`${classes.block} ${classes.filled}`}
                  aria-label={game.title}
                  onMouseEnter={() => setFocused(game)}
                  onFocus={() => setFocused(game)}
                  onClick={() => openGame(game)}
                >
                  <span className={classes.blockNum}>{num}</span>
                  <span className={classes.coverWrap}>
                    <Cover path={game.cover_path} title={game.title} />
                  </span>
                </button>
              );
            })}
          </div>
        )}

        <div className={classes.readout} aria-live="polite">
          {focused ? (
            <>
              <span className={classes.readoutTitle}>{focused.title}</span>
              <span className={classes.readoutMeta}>{platformsOf(focused)}</span>
            </>
          ) : (
            <span className={classes.readoutHint}>
              {all.length === 0 ? t('easterEggs.slotOne.empty') : t('easterEggs.slotOne.hint')}
            </span>
          )}
        </div>
      </div>
    </Modal>
  );
}
