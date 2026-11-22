import { Text, Tooltip } from '@mantine/core';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import classes from './DonutChart.module.css';

export type Slice = { key: string; label: string; value: number };

/** Past this many parts the smallest fold into one "Other" slice. */
const MAX_SLICES = 6;
const INNER = 0.62;
/** Space between slices, in viewBox units (about 2px at the drawn size). */
const GAP = 0.028;

function point(angle: number, radius: number) {
  return `${(Math.sin(angle) * radius).toFixed(4)} ${(-Math.cos(angle) * radius).toFixed(4)}`;
}

/** The angles of an arc at this radius, pulled in to leave an even gap on each side. */
function inset(start: number, end: number, radius: number): [number, number] {
  const pull = GAP / 2 / radius;
  if (end - start <= pull * 2) return [(start + end) / 2, (start + end) / 2];
  return [start + pull, end - pull];
}

/** A ring segment between two angles, clockwise from 12 o'clock. */
function arc(start: number, end: number) {
  const [os, oe] = inset(start, end, 1);
  const [is, ie] = inset(start, end, INNER);
  return [
    `M ${point(os, 1)}`,
    `A 1 1 0 ${oe - os > Math.PI ? 1 : 0} 1 ${point(oe, 1)}`,
    `L ${point(ie, INNER)}`,
    `A ${INNER} ${INNER} 0 ${ie - is > Math.PI ? 1 : 0} 0 ${point(is, INNER)}`,
    'Z',
  ].join(' ');
}

/** Each part's share of the whole, largest first, with a legend that names and counts every part. */
export function DonutChart({
  slices,
  figure,
  locale,
  empty,
}: {
  slices: Slice[];
  figure: (value: number) => string;
  locale: string;
  empty: string;
}) {
  const { t } = useTranslation();
  const [active, setActive] = useState<string | null>(null);

  const sorted = slices.filter((s) => s.value > 0).sort((a, b) => b.value - a.value);
  const parts =
    sorted.length > MAX_SLICES
      ? [
          ...sorted.slice(0, MAX_SLICES - 1),
          {
            key: 'other',
            label: t('stats.other'),
            value: sorted.slice(MAX_SLICES - 1).reduce((sum, s) => sum + s.value, 0),
          },
        ]
      : sorted;
  const total = parts.reduce((sum, s) => sum + s.value, 0);
  if (total === 0) return <Text size="sm" c="dimmed">{empty}</Text>;

  const percent = new Intl.NumberFormat(locale, { style: 'percent', maximumFractionDigits: 0 });
  const share = (value: number) => (value / total < 0.005 ? `<${percent.format(0.01)}` : percent.format(value / total));
  const color = (key: string, index: number) => `var(--series-${key === 'other' ? 'other' : index + 1})`;
  const shown = parts.reduce<(Slice & { start: number; end: number; color: string })[]>((list, part, index) => {
    const start = list.at(-1)?.end ?? 0;
    return [...list, { ...part, start, end: start + (part.value / total) * 2 * Math.PI, color: color(part.key, index) }];
  }, []);
  const current = shown.find((s) => s.key === active);

  return (
    <div className={classes.chart}>
      <Tooltip.Floating
        label={current ? [current.label, figure(current.value), share(current.value)].join(t('common.separator')) : ''}
        disabled={!current}
      >
        <svg
          className={classes.donut}
          viewBox="-1.02 -1.02 2.04 2.04"
          aria-hidden
          data-active={current ? '' : undefined}
          onMouseLeave={() => setActive(null)}
        >
          {shown.length === 1 ? (
            <circle
              className={classes.slice}
              r={(1 + INNER) / 2}
              fill="none"
              stroke={shown[0]!.color}
              strokeWidth={1 - INNER}
              onMouseEnter={() => setActive(shown[0]!.key)}
            />
          ) : (
            shown.map((s) => (
              <path
                key={s.key}
                className={classes.slice}
                d={arc(s.start, s.end)}
                style={{ fill: s.color }}
                data-active={s.key === active ? '' : undefined}
                onMouseEnter={() => setActive(s.key)}
              />
            ))
          )}
        </svg>
      </Tooltip.Floating>
      <ul className={classes.legend}>
        {shown.map((s) => (
          <li
            key={s.key}
            className={classes.row}
            data-active={s.key === active ? '' : undefined}
            onMouseEnter={() => setActive(s.key)}
            onMouseLeave={() => setActive(null)}
          >
            <span className={classes.swatch} style={{ background: s.color }} aria-hidden />
            <span className={classes.label}>{s.label}</span>
            <span className={classes.figure}>
              {figure(s.value)}
              {t('common.separator')}
              {share(s.value)}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
