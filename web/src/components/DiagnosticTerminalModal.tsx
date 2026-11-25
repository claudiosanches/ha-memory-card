import { Modal } from '@mantine/core';
import { IconX } from '@tabler/icons-react';
import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { SECRET_THEME_ID, useAppearance } from '../styles/themes/AppearanceProvider';
import classes from './DiagnosticTerminalModal.module.css';

export type CodeToken = 'up' | 'down' | 'left' | 'right' | 'b' | 'a';

const SEQUENCE: readonly CodeToken[] = ['up', 'up', 'down', 'down', 'left', 'right', 'left', 'right', 'b', 'a'];

const SYMBOL: Record<CodeToken, string> = {
  up: '↑',
  down: '↓',
  left: '←',
  right: '→',
  b: 'B',
  a: 'A',
};

type LogEntry = { kind: 'ok' | 'error' | 'hint' | 'success'; text: string };

/** Tiny Web Audio chiptune chime for the unlock moment; silently ignored if unavailable. */
function playUnlockChime() {
  try {
    if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) return;
    const ctx = new AudioCtx();
    const notes = [523.25, 659.25, 783.99, 1046.5];
    notes.forEach((freq, i) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = 'square';
      osc.frequency.value = freq;
      const start = ctx.currentTime + i * 0.08;
      gain.gain.setValueAtTime(0.06, start);
      gain.gain.exponentialRampToValueAtTime(0.001, start + 0.16);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start(start);
      osc.stop(start + 0.17);
    });
    window.setTimeout(() => void ctx.close(), 600);
  } catch {
    // Ignore audio errors in restricted or headless browsers.
  }
}

type Props = {
  opened: boolean;
  version: string;
  onClose: () => void;
};

/** Retro CRT terminal opened by clicking the app version 3 times; unlocks Cursed 90's via a secret sequence. */
export function DiagnosticTerminalModal({ opened, version, onClose }: Props) {
  const { t } = useTranslation();
  const { isUnlocked, unlockTheme, selectTheme } = useAppearance();
  const alreadyUnlocked = isUnlocked(SECRET_THEME_ID);

  const [entered, setEntered] = useState<CodeToken[]>([]);
  const [wrong, setWrong] = useState<{ index: number; symbol: string } | null>(null);
  const [mistakes, setMistakes] = useState(0);
  const [justUnlocked, setJustUnlocked] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>(() => [
    { kind: 'ok', text: t('easterEggs.crt.prompt') },
    { kind: 'ok', text: t('easterEggs.crt.controlsHint') },
  ]);

  const pushLogs = useCallback((entries: LogEntry[]) => {
    setLogs((current) => [...current, ...entries].slice(-4));
  }, []);

  const handleToken = useCallback(
    (token: CodeToken) => {
      if (justUnlocked) return;
      const expected = SEQUENCE[entered.length];
      if (token === expected) {
        const next = [...entered, token];
        setWrong(null);
        setEntered(next);
        if (next.length === SEQUENCE.length) {
          unlockTheme(SECRET_THEME_ID);
          selectTheme(SECRET_THEME_ID);
          setJustUnlocked(true);
          playUnlockChime();
          pushLogs([
            { kind: 'success', text: t('easterEggs.crt.accepted') },
            { kind: 'success', text: t('easterEggs.crt.unlocked', { theme: t('settings.themes.cursed-90s') }) },
            { kind: 'hint', text: t('easterEggs.crt.tagline') },
          ]);
        } else {
          pushLogs([
            {
              kind: 'ok',
              text: t('easterEggs.crt.signalOk', {
                symbol: SYMBOL[token],
                step: next.length,
                total: SEQUENCE.length,
              }),
            },
          ]);
        }
        return;
      }

      const nextMistakes = mistakes + 1;
      setMistakes(nextMistakes);
      setWrong({ index: entered.length, symbol: SYMBOL[token] });
      setEntered([]);

      const hintKey =
        nextMistakes > 30
          ? 'easterEggs.crt.hintMercy'
          : nextMistakes <= 2
            ? 'easterEggs.crt.hint1'
            : nextMistakes <= 4
              ? 'easterEggs.crt.hint2'
              : 'easterEggs.crt.hint3';
      pushLogs([
        {
          kind: 'error',
          text: t('easterEggs.crt.signalError', {
            symbol: SYMBOL[token],
            step: entered.length + 1,
          }),
        },
        { kind: 'hint', text: t(hintKey) },
      ]);
    },
    [entered, justUnlocked, mistakes, pushLogs, selectTheme, t, unlockTheme],
  );

  useEffect(() => {
    if (!opened) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' || event.metaKey || event.ctrlKey || event.altKey) return;
      const keyMap: Record<string, CodeToken> = {
        ArrowUp: 'up',
        ArrowDown: 'down',
        ArrowLeft: 'left',
        ArrowRight: 'right',
        b: 'b',
        B: 'b',
        a: 'a',
        A: 'a',
      };
      const token = keyMap[event.key];
      if (token) {
        event.preventDefault();
        handleToken(token);
      } else if (event.key.length === 1 && !justUnlocked) {
        pushLogs([{ kind: 'hint', text: t('easterEggs.crt.unknownKey', { key: event.key.toUpperCase() }) }]);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleToken, justUnlocked, opened, pushLogs, t]);

  const unlocked = alreadyUnlocked || justUnlocked;

  const mercyUnlock = () => {
    unlockTheme(SECRET_THEME_ID);
    selectTheme(SECRET_THEME_ID);
    setJustUnlocked(true);
    setWrong(null);
    setEntered([...SEQUENCE]);
    playUnlockChime();
    pushLogs([
      { kind: 'success', text: t('easterEggs.crt.mercyGranted') },
      { kind: 'success', text: t('easterEggs.crt.unlocked', { theme: t('settings.themes.cursed-90s') }) },
      { kind: 'hint', text: t('easterEggs.crt.tagline') },
    ]);
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      withCloseButton={false}
      centered
      size="md"
      aria-label={t('easterEggs.crt.deviceLabel')}
      classNames={{ content: classes.modalContent, body: classes.modalBody }}
    >
      <div className={classes.bezel} data-testid="diagnostic-terminal">
        <div className={classes.topBar}>
          <div className={classes.deviceLabel}>
            <span className={`${classes.led} ${wrong ? classes.ledError : ''}`} aria-hidden />
            <span>{t('easterEggs.crt.deviceLabel')}</span>
          </div>
          <button type="button" className={classes.closeBtn} aria-label={t('easterEggs.crt.close')} onClick={onClose}>
            <IconX size={15} />
          </button>
        </div>

        <div className={classes.screen}>
          <div className={classes.screenInner}>
            <div className={classes.bootHeader}>
              <span>{t('easterEggs.crt.boot', { version })}</span>
              <span>
                {unlocked
                  ? t('easterEggs.crt.moduleUnlocked', { theme: t('settings.themes.cursed-90s') })
                  : t('easterEggs.crt.moduleLocked')}
              </span>
            </div>

            <div className={classes.slots} aria-label={t('easterEggs.crt.sequenceLabel')}>
              {SEQUENCE.map((_, index) => {
                const filled = entered[index];
                const isWrong = wrong?.index === index && !filled;
                const isNext = !justUnlocked && index === entered.length && !isWrong;
                const slotClass = [
                  classes.slot,
                  filled ? classes.slotFilled : '',
                  isWrong ? classes.slotWrong : '',
                  isNext ? classes.slotActive : '',
                ]
                  .filter(Boolean)
                  .join(' ');
                return (
                  <div key={index} className={slotClass} data-testid={`code-slot-${index}`}>
                    {filled ? SYMBOL[filled] : isWrong ? wrong.symbol : '_'}
                  </div>
                );
              })}
            </div>

            <div className={classes.log} aria-live="polite" data-testid="terminal-log">
              {logs.map((entry, idx) => (
                <div
                  key={idx}
                  className={
                    entry.kind === 'error'
                      ? classes.lineError
                      : entry.kind === 'hint'
                        ? classes.lineHint
                        : entry.kind === 'success'
                          ? classes.lineSuccess
                          : classes.lineOk
                  }
                >
                  {`> ${entry.text}`}
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className={classes.controls}>
          <div className={classes.dpad} role="group" aria-label={t('easterEggs.crt.dpadLabel')}>
            <span />
            <button type="button" className={classes.padBtn} aria-label={t('easterEggs.crt.pad.up')} onClick={() => handleToken('up')}>
              {SYMBOL.up}
            </button>
            <span />
            <button type="button" className={classes.padBtn} aria-label={t('easterEggs.crt.pad.left')} onClick={() => handleToken('left')}>
              {SYMBOL.left}
            </button>
            <span className={classes.dpadCenter} aria-hidden />
            <button type="button" className={classes.padBtn} aria-label={t('easterEggs.crt.pad.right')} onClick={() => handleToken('right')}>
              {SYMBOL.right}
            </button>
            <span />
            <button type="button" className={classes.padBtn} aria-label={t('easterEggs.crt.pad.down')} onClick={() => handleToken('down')}>
              {SYMBOL.down}
            </button>
            <span />
          </div>

          <div className={classes.abGroup}>
            <button type="button" className={classes.abBtn} aria-label={t('easterEggs.crt.pad.b')} onClick={() => handleToken('b')}>
              {SYMBOL.b}
            </button>
            <button type="button" className={classes.abBtn} aria-label={t('easterEggs.crt.pad.a')} onClick={() => handleToken('a')}>
              {SYMBOL.a}
            </button>
          </div>
        </div>

        {mistakes > 30 && !justUnlocked && (
          <div className={classes.actions}>
            <button
              type="button"
              className={`${classes.actionBtn} ${classes.actionPrimary}`}
              onClick={mercyUnlock}
            >
              {t('easterEggs.crt.mercyButton')}
            </button>
          </div>
        )}
      </div>
    </Modal>
  );
}
